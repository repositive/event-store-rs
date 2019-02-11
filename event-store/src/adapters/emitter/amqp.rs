use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::internals::forward;
use crate::store::Store;
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
use log::{debug, error, info, trace};
use serde_json::Value as JsonValue;
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
        store: Store
    ) -> Result<(), io::Error>
    where
        // TODO: Fix Sync + Clone - they shouldn't be required!
        ED: EventData + EventHandler + Debug + Send + Sync + Clone,
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
                    let parsed = serde_json::from_slice::<Event<ED>>(&message.data);

                    match parsed {
                        Ok(event) => {
                            let event_id = event.id;

                            trace!("Received event {}", event_id);

                            // TODO: Does this code need to check for event existence?
                            // let exists = await!(store.event_exists(&event_id))
                            //     .expect("Could not check event existence");

                            // if !exists {
                            // TODO: Fix clone()
                            match ED::handle_event(event.clone(), &store) {
                                Ok(_) => {
                                    trace!("Handler passed, saving event");

                                    store.update_last_handled_event_log(&event).expect(&format!(
                                        "Failed to update event log for event type {} (ID {})",
                                        ED::event_namespace_and_type(),
                                        event_id
                                    ));

                                    trace!("Ack event {}", message.delivery_tag);

                                    await!(forward(channel.basic_ack(message.delivery_tag, false)))
                                        .expect("Could not ack message");
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to handle event ID {}: {}",
                                        event_id,
                                        e.to_string()
                                    );
                                }
                            }
                            // } else {
                            //     trace!("Event ID already exists");
                            // }
                        }
                        Err(e) => {
                            trace!(
                                "Failed event payload: {}",
                                String::from_utf8(message.data.clone())
                                    .unwrap_or(String::from("(failed to decode message)"))
                            );

                            serde_json::from_slice::<JsonValue>(&message.data)
                                .map(|evt| {
                                    error!(
                                        "Failed to parse event {} (ID {}): {}",
                                        ED::event_namespace_and_type(),
                                        evt["id"],
                                        e.to_string()
                                    );
                                })
                                .unwrap_or_else(|_| {
                                    error!(
                                        "Failed to parse event {} (ID unknown): {}",
                                        ED::event_namespace_and_type(),
                                        e.to_string()
                                    );
                                });
                        }
                    }
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
