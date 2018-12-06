use crate::event::Event;
use event_store_derive_internals::EventData;
use futures::future;
use futures::future::IntoFuture;
use futures::stream::Stream;
use futures::Future;
use lapin_futures::channel::{
    BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin_futures::client::{Client, ConnectionOptions};
use lapin_futures::queue::Queue;
use lapin_futures::types::FieldTable;
use serde::Serialize;
use std::collections::HashSet;
use std::io::{self, ErrorKind};
use std::net::SocketAddr;
use std::str;
use tokio::net::TcpStream;
use tokio_core::reactor::Core;

/// Connect to AMQP
pub fn amqp_connect(
    uri: SocketAddr,
    exchange: String,
) -> impl Future<Item = Channel<TcpStream>, Error = io::Error> {
    let exchange1 = exchange.clone();

    TcpStream::connect(&uri)
        .and_then(|stream| Client::connect(stream, ConnectionOptions::default()))
        .and_then(|(client, heartbeat)| {
            trace!("Start heartbeat");

            tokio::spawn(heartbeat.map_err(|e| eprintln!("heartbeat error: {:?}", e)))
                .into_future()
                .map(|_| client)
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "spawn error"))
        })
        .and_then(move |client| {
            trace!("Set up channel");

            client.create_channel()
        })
        .and_then(move |channel| {
            trace!("Declare exchange {}", exchange1);

            channel
                .exchange_declare(
                    &exchange1,
                    &"topic",
                    ExchangeDeclareOptions {
                        durable: true,
                        ..ExchangeDeclareOptions::default()
                    },
                    FieldTable::new(),
                )
                .map(|_| channel)
        })
}

/// Create a consumer for an AMQP queue
pub fn amqp_create_consumer<H, E>(
    channel: Channel<TcpStream>,
    queue_name: String,
    exchange: String,
    handler: H,
) -> impl Future<Item = (), Error = io::Error>
where
    E: EventData,
    H: Fn(Event<E>) -> (),
{
    let event_namespace = E::event_namespace();
    let event_type = E::event_type();
    let event_name = format!("{}.{}", event_namespace, event_type);

    info!(
        "Creating consumer for event {} on exchange {}",
        event_name, exchange
    );

    amqp_bind_queue(channel, queue_name, exchange, event_name)
        .and_then(move |(channel, queue, _, exchange, _)| {
            info!("Create consumer on exchange {}", exchange);

            channel
                .basic_consume(
                    &queue,
                    &exchange,
                    BasicConsumeOptions::default(),
                    FieldTable::new(),
                )
                .map(move |stream| (channel, stream))
        })
        .and_then(move |(channel, stream)| {
            info!("Got stream for consumer");

            stream.for_each(move |message| {
                let payload = str::from_utf8(&message.data).unwrap();
                let data: Event<E> = serde_json::from_str(payload).unwrap();

                trace!("Received message with ID {}: {}", data.id, payload);

                handler(data);

                channel.basic_ack(message.delivery_tag, false)
            })
        })
        .map(|_| ())
}

/// Declare and bind an AMQP queue to an exchange
// TODO: Pass in/out options struct instead of magic strings
pub fn amqp_bind_queue(
    channel: Channel<TcpStream>,
    queue_name: String,
    exchange_name: String,
    routing_key: String,
) -> impl Future<Item = (Channel<TcpStream>, Queue, String, String, String), Error = io::Error> {
    channel
        .queue_declare(
            &queue_name,
            QueueDeclareOptions {
                durable: true,
                exclusive: false,
                auto_delete: false,
                ..QueueDeclareOptions::default()
            },
            FieldTable::new(),
        )
        .map(|queue| (queue, channel))
        .and_then(move |(queue, channel)| {
            debug!("Queue {} declared", queue_name);

            channel
                .queue_bind(
                    &queue_name,
                    &exchange_name,
                    &routing_key,
                    QueueBindOptions::default(),
                    FieldTable::new(),
                )
                .map(move |_| (channel, queue, queue_name, exchange_name, routing_key))
        })
}

/// Emit an event onto a queue
pub fn amqp_emit_event<ED>(
    channel: Channel<TcpStream>,
    queue_name: String,
    exchange: String,
    event: &Event<ED>,
) -> impl Future<Item = (), Error = io::Error>
where
    ED: EventData,
{
    let payload: Vec<u8> = serde_json::to_string(&event)
        .expect("Cant serialise event")
        .into();

    let event_namespace = ED::event_namespace();
    let event_type = ED::event_type();
    let event_name = format!("{}.{}", event_namespace, event_type);

    info!("Emitting event {} onto exchange {}", event_name, exchange);

    amqp_emit_data(channel, queue_name, exchange, event_name, payload)
}

/// Emit an event onto a queue
pub fn amqp_emit_data(
    channel: Channel<TcpStream>,
    queue_name: String,
    exchange: String,
    routing_key: String,
    payload: Vec<u8>,
) -> impl Future<Item = (), Error = io::Error> {
    debug!(
        "Emitting payload through routing key {} onto exchange {}",
        routing_key, exchange
    );

    amqp_bind_queue(channel, queue_name, exchange, routing_key)
        .and_then(move |(channel, _, _, exchange_name, routing_key)| {
            channel.basic_publish(
                &exchange_name,
                &routing_key,
                payload,
                BasicPublishOptions::default(),
                BasicProperties::default(),
            )
        })
        .map(|_| {
            trace!("Data emitted",);

            ()
        })
}
