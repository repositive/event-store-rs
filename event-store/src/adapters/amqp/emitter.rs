//! AMQP emitter implementation

use adapters::EmitterAdapter;
use event_store_derive_internals::EventData;
use futures::future::{ok as FutOk, Future};
use futures::{IntoFuture, Stream};
use lapin::channel::{
    BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel, ExchangeDeclareOptions,
    QueueDeclareOptions,
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
    client: &Client<TcpStream>,
    queue: String,
    handler: H,
) -> impl Future<Item = (), Error = ()> + Send + 'static
where
    E: EventData + 'static,
    H: Fn(&Event<E>) -> () + Send + 'static,
{
    info!("will create consumer {}", 0);

    client
        .create_channel()
        .and_then(move |channel| {
            info!("creating queue {}", queue);
            channel
                .queue_declare(&queue, QueueDeclareOptions::default(), FieldTable::new())
                .map(move |queue| (channel, queue))
        })
        .and_then(move |(channel, queue)| {
            info!("creating consumer {}", 0);
            channel
                .basic_consume(
                    &queue,
                    "",
                    BasicConsumeOptions::default(),
                    FieldTable::new(),
                )
                .map(move |stream| (channel, stream))
        })
        .and_then(move |(channel, stream)| {
            info!("got stream for consumer {}", 0);
            stream.for_each(move |message| {
                let data: Event<E> =
                    serde_json::from_str(str::from_utf8(&message.data).unwrap()).unwrap();

                trace!("Received message with ID {}", data.id);

                handler(&data);

                channel.basic_ack(message.delivery_tag, false)
            })
        })
        .map(|_| ())
        .map_err(move |err| eprintln!("got error in consumer '{}': {:?}", 0, err))
}

fn connect(uri: SocketAddr, exchange: String) -> impl Future<Item = Client<TcpStream>, Error = ()> {
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
                .map(|_| client)
        })
        .map_err(|_| ())
}

impl EmitterAdapter for AMQPEmitterAdapter {
    fn emit<'a, E: EventData + Sync>(&self, event: &Event<E>) -> Result<(), io::Error> {
        let payload: Vec<u8> = serde_json::to_string(event)
            .expect("Cant serialise event")
            .into();
        let event_type = E::event_type();
        let id = event.id;
        info!("Emitting event {} with ID {}", event_type, id);

        let _channel = self.channel.clone();
        let _exchange = self.exchange.clone();

        thread::spawn(move || {
            block_on_all(
                _channel
                    .basic_publish(
                        &_exchange,
                        &event_type,
                        payload,
                        BasicPublishOptions::default(),
                        BasicProperties::default(),
                    )
                    .and_then(move |_| {
                        info!("Event with ID {} delivered", id);

                        FutOk(())
                    }),
            )
            .expect("Publish failed")
        })
        .join()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Emit error"))
    }

    fn subscribe<'a, ED, H>(&self, handler: H) -> BoxedFuture<'a, (), ()>
    where
        ED: EventData + 'static,
        H: Fn(&Event<ED>) -> () + Send + 'static,
    {
        let event_name = ED::event_type();
        let event_namespace = ED::event_namespace();
        let queue_name = format!("{}.{}", event_namespace, event_name);

        trace!("Creating queue {}", queue_name);

        Box::new(
            connect(self.uri, self.exchange.clone())
                .and_then(|client| create_consumer(&client, queue_name, handler).map(|_| ())),
        )
    }
}
