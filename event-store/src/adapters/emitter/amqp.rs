use crate::adapters::SubscribeOptions;
use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::forward;
use crate::store::Store;
use event_store_derive_internals::EventData;
use futures::Future;
use lapin_futures::channel::{
    BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin_futures::client::{Client, ConnectionOptions};
use lapin_futures::queue::Queue;
use lapin_futures::types::FieldTable;
use log::{debug, info, trace};
use std::fmt::Debug;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_async_await::stream::StreamExt;

#[derive(Clone)]
pub struct AmqpEmitterAdapter {
    channel: Channel<TcpStream>,
    exchange: String,
    store_namespace: String,
}

impl AmqpEmitterAdapter {
    pub async fn new(
        uri: SocketAddr,
        exchange: String,
        store_namespace: String,
    ) -> Result<Self, io::Error> {
        let channel = await!(amqp_connect(uri, &exchange))?;

        Ok(Self {
            channel,
            exchange,
            store_namespace,
        })
    }

    pub async fn subscribe<ED>(
        &self,
        store: Store,
        options: SubscribeOptions,
    ) -> Result<(), io::Error>
    where
        ED: EventData + EventHandler + Debug + Send + Sync + Sized,
    {
        let self_channel = self.channel.clone();
        let self_exchange = self.exchange.clone();

        let event_namespace = ED::event_namespace();
        let event_type = ED::event_type();
        let event_name = format!("{}.{}", event_namespace, event_type);
        let queue_name = self.namespaced_event_queue_name::<ED>();

        trace!("Subscribe queue {}", queue_name);

        info!(
            "Creating consumer for event {} on queue {} on exchange {}",
            event_name, queue_name, self_exchange
        );

        let queue = await!(amqp_bind_queue(
            &self_channel,
            &queue_name,
            &self_exchange,
            &event_name
        ))
        .unwrap();

        let consumer_tag = format!("consumer-{}-{}", self_exchange, event_name);

        info!(
            "Create consumer on exchange {} with consumer tag {}",
            self_exchange, consumer_tag
        );

        let mut stream = await!(forward(
            self_channel
                .basic_consume(
                    &queue,
                    &consumer_tag,
                    BasicConsumeOptions::default(),
                    FieldTable::new(),
                )
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())),
        ))
        .unwrap();

        trace!("Before while loop");

        tokio::spawn_async(
            async move {
                while let Some(Ok(message)) = await!(stream.next()) {
                    let payload = std::str::from_utf8(&message.data).unwrap();
                    let event: Event<ED> = serde_json::from_str(payload).unwrap();

                    trace!("Received event {:?}", event);

                    let saved = if options.save_on_receive {
                        await!(store.save_no_emit(&event))
                    } else {
                        Ok(())
                    };

                    saved
                        .map(|_| {
                            ED::handle_event(event, &store);

                            self_channel.basic_ack(message.delivery_tag, false);
                        })
                        .expect("Could not save event");
                }
            },
        );

        Ok(())
    }

    pub async fn emit<'a, ED>(&'a self, event: &'a Event<ED>) -> Result<(), io::Error>
    where
        ED: EventData,
    {
        let payload: Vec<u8> = serde_json::to_string(&event)
            .expect("Cant serialise event")
            .into();

        let event_namespace = ED::event_namespace();
        let event_type = ED::event_type();
        let event_name = format!("{}.{}", event_namespace, event_type);
        let queue_name = self.namespaced_event_queue_name::<ED>();

        info!(
            "Emitting event {} onto exchange {} through queue {}",
            event_name, self.exchange, queue_name
        );

        await!(amqp_emit_data(
            &self.channel,
            &queue_name,
            &self.exchange,
            &event_name,
            payload
        ))?;

        Ok(())
    }

    fn namespaced_event_queue_name<ED>(&self) -> String
    where
        ED: EventData,
    {
        format!("{}-{}", self.store_namespace, self.event_queue_name::<ED>())
    }

    fn event_queue_name<ED>(&self) -> String
    where
        ED: EventData,
    {
        format!("{}.{}", ED::event_namespace(), ED::event_type())
    }
}

async fn amqp_connect(uri: SocketAddr, exchange: &String) -> Result<Channel<TcpStream>, io::Error> {
    let exchange1 = exchange.clone();

    let stream: TcpStream = await!(forward(TcpStream::connect(&uri)))?;

    let (client, heartbeat) = await!(forward(Client::connect(
        stream,
        ConnectionOptions::default()
    )))
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    tokio::spawn(heartbeat.map_err(|e| eprintln!("heartbeat error: {:?}", e)));

    let channel = await!(forward(client.create_channel()))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    await!(forward(channel.exchange_declare(
        &exchange1,
        &"topic",
        ExchangeDeclareOptions {
            durable: true,
            ..ExchangeDeclareOptions::default()
        },
        FieldTable::new(),
    )))
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok(channel)
}

async fn amqp_bind_queue<'a>(
    channel: &'a Channel<TcpStream>,
    queue_name: &'a String,
    exchange_name: &'a String,
    routing_key: &'a String,
) -> Result<Queue, io::Error> {
    let queue = await!(forward(channel.queue_declare(
        &queue_name,
        QueueDeclareOptions {
            durable: true,
            exclusive: false,
            auto_delete: false,
            ..QueueDeclareOptions::default()
        },
        FieldTable::new(),
    )))
    .unwrap();

    await!(forward(
        channel
            .queue_bind(
                &queue_name,
                &exchange_name,
                &routing_key,
                QueueBindOptions::default(),
                FieldTable::new(),
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    ))?;

    Ok(queue)
}

async fn amqp_emit_data<'a>(
    channel: &'a Channel<TcpStream>,
    queue_name: &'a String,
    exchange: &'a String,
    routing_key: &'a String,
    payload: Vec<u8>,
) -> Result<(), io::Error> {
    debug!(
        "Emitting payload through routing key {} onto exchange {}",
        routing_key, exchange
    );

    await!(amqp_bind_queue(
        &channel,
        &queue_name,
        &exchange,
        &routing_key
    ))?;

    await!(forward(
        channel
            .basic_publish(
                &exchange,
                &routing_key,
                payload,
                BasicPublishOptions::default(),
                BasicProperties::default(),
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    ))?;

    Ok(())
}
