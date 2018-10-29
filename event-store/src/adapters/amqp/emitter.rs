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
                // .and_then(|(client, channel)| {
                //     let _client = client.clone();
                //     trace!("Create consumer");
                //     create_consumer(&_client)
                //         .map_err(|_| io::Error::new(io::ErrorKind::Other, "spawn error"))
                //         .map(|_| (client, channel))
                // })
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

fn prepare_subscription<'a, E, H>(
    exchange: String,
    handler: H,
    client: Client<TcpStream>,
) -> impl Future<Item = (), Error = ()>
where
    E: EventData + 'a,
    H: Fn(&Event<E>) -> () + Send + 'static,
{
    let event_name = E::event_type();
    let event_namespace = E::event_namespace();
    // let queue_name = format!("{}-{}", event_namespace, event_name);
    let queue_name = "organisations.MembershipEdited";
    // let c_channel = channel.clone();
    let queue_name1 = queue_name.clone();
    trace!("prepare_subscription {}", queue_name);

    client
        .create_channel()
        .and_then(move |channel| {
            info!("creating channel {}", queue_name);
            channel
                .queue_declare(
                    &queue_name,
                    QueueDeclareOptions::default(),
                    FieldTable::new(),
                )
                .map(move |queue| (channel, queue))
        })
        // .and_then(move |(channel, queue)| {
        //     info!("Binding queue {} to exchange {}", queue_name, exchange);
        //     channel
        //         .queue_bind(
        //             &queue_name,
        //             &exchange,
        //             &event_name,
        //             QueueBindOptions::default(),
        //             FieldTable::new(),
        //         )
        .and_then(move |(channel, queue)| {
            trace!("Basic_consume");

            channel
                .basic_consume(
                    &queue,
                    &queue_name,
                    BasicConsumeOptions::default(),
                    FieldTable::new(),
                )
                .map(move |stream| (channel, stream))
        })
        .and_then(move |(channel, stream)| {
            info!("Starting to consume from queue {}", queue_name1);

            stream
                .for_each(move |message| {
                    trace!("MESSAGE {}", str::from_utf8(&message.data).unwrap());

                    let data: Event<E> =
                        serde_json::from_str(str::from_utf8(&message.data).unwrap()).unwrap();
                    info!("Receiving message with id {}", data.id);
                    handler(&data);
                    channel.basic_ack(message.delivery_tag, false)
                })
                .map_err(|e| {
                    panic!(e);
                })
        })
        .map(|_| ())
        // .map_err(|e| e.into())
        .map_err(|_| ())
}

fn create_consumer(
    client: &Client<TcpStream>,
) -> impl Future<Item = (), Error = ()> + Send + 'static {
    info!("will create consumer {}", 0);

    let queue = format!("organisations.MembershipEdited");

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
                println!(
                    "consumer '{}' got '{}'",
                    0,
                    str::from_utf8(&message.data).unwrap()
                );
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
                .map(|_| client)
        })
        .map_err(|_| ())
    // .and_then(|(client, channel)| {
    //     let _client = client.clone();
    //     trace!("Create consumer");
    //     create_consumer(&_client)
    //         .map_err(|_| io::Error::new(io::ErrorKind::Other, "spawn error"))
    // })
    // .map(|_| ())
    // // .map_err(|e| e.into())
    // .map_err(|_| ())
}

impl EmitterAdapter for AMQPEmitterAdapter {
    fn emit<'a, E: EventData + Sync>(&self, event: &Event<E>) -> BoxedFuture<'a, (), io::Error> {
        let payload: Vec<u8> = serde_json::to_string(event)
            .expect("Cant serialise event")
            .into();
        let event_type = E::event_type();
        let id = event.id;
        info!("Emitting event {} with id {}", event_type, id);

        Box::new(
            self.channel
                .basic_publish(
                    &self.exchange,
                    &event_type,
                    payload,
                    BasicPublishOptions::default(),
                    BasicProperties::default(),
                )
                .and_then(move |_| {
                    info!("Event with id {} delivered", id);
                    FutOk(())
                }),
        )
    }

    fn subscribe<'a, ED, H>(&self, handler: H) -> BoxedFuture<'a, (), ()>
    where
        ED: EventData + 'a + 'static,
        H: Fn(&Event<ED>) -> () + Send + 'static,
    {
        // Box::new(prepare_subscription(
        //     self.exchange.clone(),
        //     handler,
        //     self.client.clone(),
        // ))

        // trace!("BEF");
        // block_on_all(create_consumer(&self.client));
        // trace!("AFT");

        Box::new(
            // prepare_subscription(self.exchange.clone(), handler, self.client.clone())
            //     .into_future()
            //     .map_err(|_| ()),
            connect(self.uri, self.exchange.clone())
                .and_then(|client| create_consumer(&client))
                .map(|_| ())
                .map_err(|_| ()),
        )
    }
}
