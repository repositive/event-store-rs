//! AMQP emitter implementation

use adapters::EmitterAdapter;
use event_store_derive_internals::EventData;
use futures::future::{ok as FutOk, Future};
use futures::{IntoFuture, Stream};
use lapin::channel::{
    BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::client::{Client, ConnectionOptions};
use lapin::types::FieldTable;
use serde_json;
use std::io;
use std::net::SocketAddr;
use std::str;
use std::thread::{self, JoinHandle};
use tokio;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use utils::BoxedFuture;
use Event;

/// AMQP emitter
#[derive(Clone)]
pub struct AMQPEmitterAdapter {
    channel: Channel<TcpStream>,
    client: Client<TcpStream>,
    exchange: String,
    uri: SocketAddr,
}

impl AMQPEmitterAdapter {
    /// Create a new AMQPEmiterAdapter
    pub fn new<'a>(uri: SocketAddr, exchange: String) -> BoxedFuture<'a, Self, io::Error> {
        let exchange1 = exchange.clone();
        info!("Connecting to AMQP using {}", uri);
        Box::new(
            TcpStream::connect(&uri)
                .and_then(|stream| Client::connect(stream, ConnectionOptions::default()))
                .and_then(|(client, heartbeat)| {
                    trace!("Start heartbeat");

                    tokio::spawn(heartbeat.map_err(|e| error!("Heartbeat spawn error: {:?}", e)))
                        .into_future()
                        .map(|_| client)
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Heartbeat spawn error"))
                })
                .and_then(move |client| {
                    trace!("Set up channel");

                    client
                        .create_channel()
                        .map(move |channel| (client, channel))
                })
                .and_then(move |(client, channel)| {
                    trace!("Exchange declare");

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
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Channel spawn error"))
                        .map(|_| (client, channel))
                })
                .and_then(move |(client, channel)| {
                    trace!("Channel created");

                    FutOk(Self {
                        client,
                        channel,
                        exchange,
                        uri,
                    })
                }),
        )
    }
}

fn create_consumer<H, E>(
    channel: Channel<TcpStream>,
    queue_name: String,
    exchange: String,
    handler: H,
) -> impl Future<Item = (), Error = ()> + Send + 'static
where
    E: EventData + 'static,
    H: Fn(Event<E>) -> () + Send + 'static,
{
    let event_namespace = E::event_namespace();
    let event_type = E::event_type();
    let event_name = format!("{}.{}", event_namespace, event_type);

    info!(
        "Creating consumer for event {} on exchange {}",
        event_name, exchange
    );

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
        .map(|queue| (channel, queue))
        .and_then(move |(channel, queue)| {
            trace!("Bind queue");

            channel
                .queue_bind(
                    &event_name,
                    &exchange,
                    &event_name,
                    QueueBindOptions::default(),
                    FieldTable::new(),
                )
                .map(|_| (channel, queue, exchange))
        })
        .and_then(move |(channel, queue, exchange)| {
            info!("Create consumer");

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
        .map_err(move |err| error!("got error in consumer {:?}", err))
}

fn connect(
    uri: SocketAddr,
    exchange: String,
) -> impl Future<Item = (Client<TcpStream>, Channel<TcpStream>), Error = ()> {
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

            client
                .create_channel()
                .map(move |channel| (client, channel))
        })
        .and_then(move |(client, channel)| {
            trace!("Exchange declare");

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
                .map(|_| (client, channel))
        })
        .map_err(|e| panic!("Shiet {:?}", e))
}

impl EmitterAdapter for AMQPEmitterAdapter {
    fn emit<E: EventData>(&self, event: &Event<E>) -> Result<(), io::Error> {
        let payload: Vec<u8> = serde_json::to_string(event)
            .expect("Cant serialise event")
            .into();
        let event_namespace = E::event_namespace();
        let event_type = E::event_type();
        let id = event.id;

        let event_name = format!("{}.{}", event_namespace, event_type);

        trace!(
            "Emit event {} (ID {}) to exchange {}",
            event_name,
            id,
            self.exchange
        );

        let fut = self
            .channel
            .basic_publish(
                &self.exchange,
                &event_name,
                payload,
                BasicPublishOptions::default(),
                BasicProperties::default(),
            )
            .map(|_| {
                trace!("Got to end");
            });

        Runtime::new().unwrap().block_on(fut)
    }

    fn subscribe<ED, H>(&self, handler: H) -> JoinHandle<()>
    where
        ED: EventData + 'static,
        H: Fn(Event<ED>) -> () + Send + 'static,
    {
        let event_name = ED::event_type();
        let event_namespace = ED::event_namespace();
        let queue_name = format!("{}.{}", event_namespace, event_name);
        let exchange = self.exchange.clone();

        trace!("Creating queue {}", queue_name);

        thread::spawn(move || {
            let consumer = connect(self.uri, self.exchange.clone())
                .and_then(|(_, channel)| create_consumer(channel, queue_name, exchange, handler));

            trace!("Begin listen");

            Runtime::new().unwrap().block_on_all(consumer).unwrap();
        })
    }
}
