use crate::adapters::SaveStatus;
use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::internals::forward;
use crate::store::Store;
use crate::subscribe_options::SubscribeOptions;
use event_store_derive_internals::EventData;
use futures::Future;
use lapin_futures::channel::{
    BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin_futures::client::{Client, ConnectionOptions};
use lapin_futures::consumer::Consumer;
use lapin_futures::queue::Queue;
use lapin_futures::types::FieldTable;
use log::{debug, info, trace};
use serde::Serialize;
use std::fmt::Debug;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_async_await::stream::StreamExt;

/// AMQP-backed emitter/subscriber
#[derive(Clone)]
pub struct AmqpEmitterAdapter {
    channel: Channel<TcpStream>,
    exchange: String,
    store_namespace: String,
    uri: SocketAddr,
}

impl AmqpEmitterAdapter {
    /// Create a new AMQP emitter/subscriber
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
            uri,
        })
    }

    /// Subscribe to an event
    pub async fn subscribe<ED>(
        &self,
        store: Store,
        options: SubscribeOptions,
    ) -> Result<(), io::Error>
    where
        ED: EventData + EventHandler + Debug + Send,
    {
        let channel = await!(amqp_connect(self.uri, &self.exchange))?;

        let event_namespace = ED::event_namespace();
        let event_type = ED::event_type();
        let event_name = format!("{}.{}", event_namespace, event_type);
        let queue_name = self.namespaced_event_queue_name::<ED>();

        trace!("Subscribe queue {}", queue_name);

        let queue = await!(amqp_bind_queue(
            &channel,
            &queue_name,
            &self.exchange,
            &event_name
        ))
        .unwrap();

        info!(
            "Creating consumer for event {} on queue {} on exchange {}",
            event_name, queue_name, self.exchange
        );

        let mut stream: Consumer<TcpStream> = await!(forward(
            channel
                .basic_consume(
                    &queue,
                    &"",
                    BasicConsumeOptions::default(),
                    FieldTable::new(),
                )
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())),
        ))
        .unwrap();

        // TODO: Move this logic out into subscribable_store to dedupe it from backing stores
        tokio::spawn_async(
            async move {
                while let Some(Ok(message)) = await!(stream.next()) {
                    let event: Event<ED> = serde_json::from_slice(&message.data).unwrap();

                    trace!("Received event {}", event.id);

                    let saved = if options.save_on_receive {
                        trace!(
                            "Save event {} ({}.{})",
                            event.id,
                            ED::event_namespace(),
                            ED::event_type()
                        );

                        store.save_no_emit(&event)
                    } else {
                        trace!(
                            "Skip saving event {} ({}.{})",
                            event.id,
                            ED::event_namespace(),
                            ED::event_type()
                        );

                        Ok(SaveStatus::Ok)
                    };

                    // TODO: Check order of save/handle or handle/save based on TS event store
                    saved
                        .map(|result| match result {
                            SaveStatus::Ok => {
                                trace!("Event saved, calling handler");
                                ED::handle_event(event, &store);
                            }
                            SaveStatus::Duplicate => {
                                debug!("Duplicate event {}, skipping handler", event.id);
                            }
                        })
                        .expect("Failed to handle event");

                    trace!("Ack event {}", message.delivery_tag);

                    await!(forward(channel.basic_ack(message.delivery_tag, false)))
                        .expect("Could not ack message");
                }
            },
        );

        Ok(())
    }

    /// Emit an event
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
            &self.exchange,
            &event_name,
            payload
        ))?;

        Ok(())
    }

    pub(crate) async fn emit_value<'a, V>(
        &'a self,
        event_namespace: &'a str,
        event_type: &'a str,
        data: &'a V,
    ) -> Result<(), io::Error>
    where
        V: Serialize,
    {
        let payload: Vec<u8> = serde_json::to_string(&data)
            .expect("Cant serialise data")
            .into();

        let event_name = format!("{}.{}", event_namespace, event_type);
        let queue_name = format!("{}-{}", self.store_namespace, event_name);

        info!(
            "Emitting data {} onto exchange {} through queue {}",
            event_name, self.exchange, queue_name
        );

        await!(amqp_emit_data(
            &self.channel,
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
        ConnectionOptions {
            frame_max: 65535,
            heartbeat: 20,
            ..ConnectionOptions::default()
        }
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
    debug!(
        "Bind queue {} to exchange {} through routing key {}",
        queue_name, exchange_name, routing_key
    );

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
    exchange: &'a String,
    routing_key: &'a String,
    payload: Vec<u8>,
) -> Result<(), io::Error> {
    debug!(
        "Emitting payload through routing key {} onto exchange {}",
        routing_key, exchange
    );

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
