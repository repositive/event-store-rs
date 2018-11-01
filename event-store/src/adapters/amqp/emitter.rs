//! AMQP emitter implementation

use adapters::EmitterAdapter;
use event_store_derive_internals::EventData;
use futures;
use futures::future::{ok as FutOk, Future};
use futures::lazy;
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
use std::thread;
use tokio;
use tokio::net::TcpStream;
use tokio::runtime::current_thread::block_on_all;
use tokio::runtime::Runtime;
use utils::BoxedFuture;
use Event;

fn create_consumer<H, E>(
    channel: Channel<TcpStream>,
    queue_name: String,
    handler: H,
) -> impl Future<Item = (), Error = ()> + Send + 'static
where
    E: EventData + 'static,
    H: Fn(&Event<E>) -> () + Send + 'static,
{
    let event_namespace = E::event_namespace();
    let event_type = E::event_type();

    let event_name = format!("{}.{}", event_namespace, event_type);

    // let _channel = channel.clone();
    // let _queue = queue_name.clone();

    info!("will create consumer for {}", event_name);

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
        // .and_then(|_| {
        //     trace!("ONE");
        //     FutOk(())
        // })
        .map(|queue| (channel, queue))
        .and_then(move |(channel, queue)| {
            trace!("TWO");
            channel
                .queue_bind(
                    &event_name,
                    // TODO: Pass in
                    "iris",
                    &event_name,
                    QueueBindOptions::default(),
                    FieldTable::new(),
                )
                .map(|_| (channel, queue))
        })
        .and_then(move |(channel, queue)| {
            info!("creating consumer {}", 0);
            channel
                .basic_consume(
                    &queue,
                    // TODO: Pass in
                    "iris",
                    BasicConsumeOptions::default(),
                    FieldTable::new(),
                )
                .map(move |stream| (channel, stream))
        })
        .and_then(move |(channel, stream)| {
            info!("got stream for consumer {}", 0);
            stream.for_each(move |message| {
                let payload = str::from_utf8(&message.data).unwrap();
                let data: Event<E> = serde_json::from_str(payload).unwrap();
                trace!("Received message with ID {}: {}", data.id, payload);
                handler(&data);
                channel.basic_ack(message.delivery_tag, false)
            })
        })
        .map(|_| ())
        .map_err(move |err| error!("got error in consumer {:?}", err))
}

fn prepare_subscription<'a, E, H>(
    exchange: String,
    handler: H,
    channel: Channel<TcpStream>,
) -> impl Future<Item = (), Error = io::Error>
where
    E: EventData + 'a,
    H: Fn(&Event<E>) -> () + Send + 'static,
{
    let event_name = E::event_type();
    let event_namespace = E::event_namespace();
    let queue_name = format!("{}.{}", event_namespace, event_name);
    let c_channel = channel.clone();
    let queue_name1 = queue_name.clone();
    info!("Creating queue {}", queue_name);
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
        .and_then(move |queue| {
            info!("Binding queue {} to exchange {}", queue_name, exchange);
            channel
                .queue_bind(
                    &queue_name,
                    &exchange,
                    &event_name,
                    QueueBindOptions::default(),
                    FieldTable::new(),
                )
                .and_then(move |_| {
                    channel.basic_consume(
                        &queue,
                        &queue_name,
                        BasicConsumeOptions::default(),
                        FieldTable::new(),
                    )
                })
                .and_then(move |stream| {
                    let handle_events = stream
                        .for_each(move |message| {
                            let data: Event<E> =
                                serde_json::from_str(str::from_utf8(&message.data).unwrap())
                                    .unwrap();
                            info!("Receiving message with id {}", data.id);
                            handler(&data);
                            c_channel.basic_ack(message.delivery_tag, false)
                        })
                        .map_err(|e| {
                            panic!(e);
                        });

                    info!("Starting to consume from queue {}", queue_name1);
                    tokio::spawn(handle_events);
                    FutOk(())
                })
        })
        .and_then(|_| FutOk(()))
        .map_err(|e| e.into())
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

/// TODO: DOcument this biatch
#[derive(Clone)]
pub struct AMQPSender {
    exchange: String,
    channel: Channel<TcpStream>,
}

impl AMQPSender {
    /// TODO: DOcument this biatch
    pub fn new(exchange: String, channel: Channel<TcpStream>) -> Self {
        Self { exchange, channel }
    }

    /// TODO: DOcument this biatch
    pub fn emit<'a, E: EventData + Sync>(&self, event: &Event<E>) -> Result<(), io::Error> {
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

        block_on_all(fut)
    }
}

/// TODO: DOcument this biatch
#[derive(Clone)]
pub struct AMQPReceiver {
    exchange: String,
    uri: SocketAddr,
}

impl AMQPReceiver {
    /// TODO: DOcument this biatch
    pub fn new(exchange: String, uri: SocketAddr) -> Self {
        Self { exchange, uri }
    }

    /// TODO: DOcument this biatch
    pub fn subscribe<'a, ED, H>(&self, handler: H)
    where
        ED: EventData + 'static,
        H: Fn(&Event<ED>) -> () + Send + 'static,
    {
        let event_name = ED::event_type();
        let event_namespace = ED::event_namespace();
        let queue_name = format!("{}.{}", event_namespace, event_name);

        trace!("Creating queue {}", queue_name);

        let consumer = connect(self.uri, self.exchange.clone())
            .and_then(|(_, channel)| create_consumer(channel, queue_name, handler));

        tokio::run(futures::lazy(|| {
            trace!("Subscribe");

            tokio::spawn(consumer);

            Ok(())
        }));
    }
}

/// AMQP emitter
#[derive(Clone)]
pub struct AMQPEmitterAdapter {
    // channel: Channel<TcpStream>,
    // client: Client<TcpStream>,
    // exchange: String,
    // uri: SocketAddr,
    sender: AMQPSender,
    receiver: AMQPReceiver,
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
                    trace!("heartbeat");
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
                        .map_err(|_| io::Error::new(io::ErrorKind::Other, "spawn error"))
                        .map(|_| (client, channel))
                })
                .and_then(move |(client, channel)| {
                    trace!("Channel created");

                    FutOk(Self {
                        // TODO: This should be a str and just copied or something
                        sender: AMQPSender::new(exchange.clone(), channel),
                        // TODO: This should be a str and just copied or something
                        receiver: AMQPReceiver::new(exchange.clone(), uri),
                        // client,
                        // channel,
                        // exchange,
                        // uri,
                    })
                }),
        )
    }
}

impl EmitterAdapter for AMQPEmitterAdapter {
    fn emit<'a, E: EventData + Sync>(&self, event: &Event<E>) -> Result<(), io::Error> {
        self.sender.emit(event)
    }

    fn subscribe<'a, ED, H>(&self, handler: H)
    where
        ED: EventData + 'static,
        H: Fn(&Event<ED>) -> () + Send + 'static,
    {
        self.receiver.subscribe(handler)
    }

    /// Split into sender and receiver
    fn split(self) -> (AMQPReceiver, AMQPSender) {
        (self.receiver, self.sender)
    }
}
