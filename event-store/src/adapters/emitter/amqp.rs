//! AMQP emitter implementation

use super::EmitterAdapter;
use event::Event;
use event_store_derive_internals::EventData;
use futures::future::{ok as FutOk, Future};
use futures::{IntoFuture, Stream};
use lapin::channel::{
    BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::client::{Client, ConnectionOptions};
use lapin::consumer::Consumer;
use lapin::types::FieldTable;
use serde_json;
use serde_json::{to_value, Value as JsonValue};
use std::io;
use std::net::SocketAddr;
use std::str;
use std::sync::mpsc::channel;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio;
use tokio::net::TcpStream;
use tokio::runtime::current_thread::block_on_all;
use tokio::runtime::current_thread::Runtime as CurrentThreadRuntime;
use tokio::runtime::Runtime;
use utils::BoxedFuture;

/// AMQP emitter
#[derive(Clone)]
pub struct AMQPEmitterAdapter {
    // channel: Channel<TcpStream>,
    // client: Client<TcpStream>,
    exchange: String,
    uri: SocketAddr,
    channel: Channel<TcpStream>,
}

impl AMQPEmitterAdapter {
    /// Create a new AMQPEmiterAdapter
    // pub fn new<'a>(uri: SocketAddr, exchange: String) -> BoxedFuture<'a, Self, io::Error> {
    //     let exchange1 = exchange.clone();
    //     info!("Connecting to AMQP using {}", uri);
    //     Box::new(
    //         TcpStream::connect(&uri)
    //             .and_then(|stream| Client::connect(stream, ConnectionOptions::default()))
    //             .and_then(|(client, heartbeat)| {
    //                 trace!("Start heartbeat");

    //                 tokio::spawn(heartbeat.map_err(|e| error!("Heartbeat spawn error: {:?}", e)))
    //                     .into_future()
    //                     .map(|_| client)
    //                     .map_err(|_| io::Error::new(io::ErrorKind::Other, "Heartbeat spawn error"))
    //             })
    //             .and_then(move |client| {
    //                 trace!("Set up channel");

    //                 client
    //                     .create_channel()
    //                     .map(move |channel| (client, channel))
    //             })
    //             .and_then(move |(client, channel)| {
    //                 trace!("Exchange declare");

    //                 channel
    //                     .exchange_declare(
    //                         &exchange1,
    //                         &"topic",
    //                         ExchangeDeclareOptions {
    //                             durable: true,
    //                             ..ExchangeDeclareOptions::default()
    //                         },
    //                         FieldTable::new(),
    //                     )
    //                     .map_err(|_| io::Error::new(io::ErrorKind::Other, "Channel spawn error"))
    //                     .map(|_| (client, channel))
    //             })
    //             .and_then(move |(_client, channel)| {
    //                 trace!("Channel created");

    //                 FutOk(Self {
    //                     // client,
    //                     channel,
    //                     exchange,
    //                     uri,
    //                 })
    //             }),
    //     )
    // }
    pub fn new(uri: SocketAddr, exchange: String) -> Self {
        let channel = connect(uri.clone(), exchange.clone());

        let channel = Runtime::new().unwrap().block_on(channel).unwrap();

        Self {
            exchange,
            uri,
            channel: channel.1,
        }
    }
}

// TODO: Pass args as refs
fn create_consumer<ED>(
    channel: Channel<TcpStream>,
    queue_name: String,
    exchange: String,
    // handler: H,
) -> impl Future<Item = (Channel<TcpStream>, Consumer<TcpStream>), Error = ()> + Send + 'static
where
    ED: EventData + 'static,
    // H: Fn(Event<ED>) -> () + Send + 'static,
{
    let event_namespace = ED::event_namespace();
    let event_type = ED::event_type();
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
        .map_err(|_| error!("Got error in consumer"))
    // .and_then(move |(channel, stream)| {
    //     info!("Got stream for consumer");

    //     stream.for_each(move |message| {
    //         let payload = str::from_utf8(&message.data).unwrap();
    //         let data: Event<ED> = serde_json::from_str(payload).unwrap();
    //         trace!("Received message with ID {}: {}", data.id, payload);
    //         handler(data);
    //         channel.basic_ack(message.delivery_tag, false)
    //     })
    // })
    // .map(|_| ())
    // .map_err(move |err| error!("got error in consumer {:?}", err))
}

// TODO: Pass args as refs
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
    fn emit<E: EventData>(&self, event: &Event<E>) -> BoxedFuture<(), ()> {
        let event_namespace = E::event_namespace();
        let event_type = E::event_type();
        let event_name = format!("{}.{}", event_namespace, event_type);

        trace!(
            "Emit event {} (ID {}) to exchange {}",
            event_name,
            event.id,
            self.exchange
        );

        self.emit_with_string_ident(
            event_namespace.into(),
            event_type.into(),
            &to_value(event).unwrap(),
        )
    }

    fn emit_with_string_ident(
        &self,
        event_namespace: &str,
        event_type: &str,
        event: &JsonValue,
    ) -> BoxedFuture<(), ()> {
        let payload: Vec<u8> = serde_json::to_string(event)
            .expect("Cant serialise event")
            .into();

        let event_name = format!("{}.{}", event_namespace, event_type);

        trace!("Emit event {} to exchange {}", event_name, self.exchange);

        let _exchange = self.exchange.clone();

        let fut = self
            .channel
            .basic_publish(
                &_exchange,
                &event_name,
                payload,
                BasicPublishOptions::default(),
                BasicProperties::default(),
            )
            .map(|_| ())
            .map_err(|_| ());

        Box::new(fut)

        // tokio::spawn(fut);

        // // CurrentThreadRuntime::new().unwrap().block_on(fut)

        // Ok(())
    }

    fn subscribe<ED, H>(&self, handler: H) -> BoxedFuture<(), ()>
    where
        ED: EventData + 'static,
        H: Fn(Event<ED>) -> () + Send + 'static,
    {
        let event_name = ED::event_type();
        let event_namespace = ED::event_namespace();
        let queue_name = format!("{}.{}", event_namespace, event_name);
        let _queue_name = queue_name.clone();
        let _exchange = self.exchange.clone();
        let _uri = self.uri;

        trace!("Creating queue {}", queue_name);

        let consumer = connect(_uri, _exchange.clone())
            .and_then(|(_, channel)| create_consumer::<ED>(channel, queue_name, _exchange))
            .and_then(move |(channel, stream)| {
                trace!("Begin consuming stream");

                let consume = stream
                    .for_each(move |message| {
                        let payload = str::from_utf8(&message.data).unwrap();
                        let data: Event<ED> = serde_json::from_str(payload).unwrap();
                        trace!("Received message with ID {}: {}", data.id, payload);
                        handler(data);
                        channel.basic_ack(message.delivery_tag, false)
                    })
                    .map_err(|_| ());

                tokio::spawn(consume);

                trace!("Consumer for {} spawned", _queue_name);

                // consume

                FutOk(())
            });

        // tokio::spawn(consumer);

        // thread::spawn(|| {
        //     // tokio::run(consumer);

        //     trace!("Sub noop");
        // })

        Box::new(consumer)

        // let mut rt = CurrentThreadRuntime::new().unwrap();
        // let handle = rt.handle();

        // let consumer = connect(_uri, _exchange.clone())
        //     .and_then(|(_, channel)| create_consumer::<ED>(channel, queue_name, _exchange));

        // let (tx, rx) = channel();

        // let (channel, stream) = rt.block_on(consumer).unwrap();

        // let handle = thread::spawn(move || {
        //     trace!("Begin listen");

        //     let stream_fut = stream.for_each(move |message| {
        //         let payload = str::from_utf8(&message.data).unwrap();
        //         let data: Event<ED> = serde_json::from_str(payload).unwrap();
        //         let id = data.id;
        //         debug!("Received message with ID {}", id);
        //         // trace!("Message ID {} payload: {}", id, payload);
        //         handler(data);
        //         trace!(
        //             "Handler passed, acking {} for event ID {}",
        //             message.delivery_tag,
        //             id
        //         );

        //         channel.basic_ack(message.delivery_tag, false)
        //     });

        //     tx.send(()).unwrap();

        //     tokio::spawn(stream_fut.into_future());

        //     // CurrentThreadRuntime::new()
        //     //     .unwrap()
        //     //     .block_on(stream_fut)
        //     //     .unwrap();
        // });

        // rx.recv().unwrap();

        // handle
    }
}
