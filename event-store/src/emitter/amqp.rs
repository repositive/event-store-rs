use crate::emitter::EmitterAdapter;
use crate::emitter::EmitterReceiver;
use crate::emitter::EmitterSender;
use crate::{Event, Store};
use event_store_derive_internals::EventData;
use futures::future::{self, ok as FutOk, Future, IntoFuture};
use futures::stream::Stream;
use lapin_futures::channel::{
    BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin_futures::client::{Client, ConnectionOptions};
use lapin_futures::types::FieldTable;
use std::fmt;
use std::io;
use std::net::SocketAddr;
use std::str;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};
use tokio::net::TcpStream;
use tokio::runtime::current_thread::block_on_all;
use tokio::runtime::Runtime;
// use tokio::runtime::current_thread::Runtime;

#[derive(Debug)]
pub struct AMQPEmitterAdapter {
    sender: AMQPSender,
    receiver: AMQPReceiver,
}

impl AMQPEmitterAdapter {
    pub fn new(
        uri: SocketAddr,
        exchange: String,
    ) -> Box<Future<Item = Self, Error = io::Error> + Send> {
        Box::new(
            AMQPSender::new(uri, exchange.clone()).and_then(move |sender| {
                FutOk(Self {
                    sender,
                    receiver: AMQPReceiver::new(uri, exchange.clone()),
                })
            }),
        )
    }
}

impl<ED, TX, RX> EmitterAdapter<ED, TX, RX> for AMQPEmitterAdapter
where
    ED: EventData,
    TX: EmitterSender<ED>,
    RX: EmitterReceiver<ED>,
{
    fn split(self) -> (TX, RX) {
        (self.sender, self.receiver)
    }
}

#[derive(Clone)]
pub struct AMQPSender {
    uri: SocketAddr,
    exchange: String,
    channel: Channel<TcpStream>,
}

impl fmt::Debug for AMQPSender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AMQPSender {{ uri: {:?}, exchange: {:?} }}",
            self.uri, self.exchange
        )
    }
}

impl AMQPSender {
    pub fn new(
        uri: SocketAddr,
        exchange: String,
    ) -> Box<Future<Item = Self, Error = io::Error> + Send> {
        let _exchange = exchange.clone();

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
                            &_exchange,
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
                .and_then(move |(_client, channel)| {
                    trace!("Channel created");

                    FutOk(Self {
                        channel,
                        exchange,
                        uri,
                    })
                }),
        )
    }
}

impl<ED> EmitterSender<ED> for AMQPSender
where
    ED: EventData,
{
    fn emit(&self, event: &Event<ED>) {
        let payload: Vec<u8> = serde_json::to_string(event)
            .expect("Cant serialise event")
            .into();
        let id = event.id;

        let event_name = "queue_name_here";

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

        Runtime::new()
            .expect("Emit runtime")
            .block_on(fut)
            .expect("Emit future");
    }
}

#[derive(Debug)]
pub struct AMQPReceiver {
    uri: SocketAddr,
    exchange: String,
}

impl AMQPReceiver {
    pub fn new(uri: SocketAddr, exchange: String) -> Self {
        Self { uri, exchange }
    }
}

impl<ED> EmitterReceiver<ED> for AMQPReceiver
where
    ED: EventData,
{
    fn subscribe<H>(&self, handler: H) -> JoinHandle<()>
    where
        H: Fn(Event<ED>) -> () + Send + 'static,
    {
        let _uri = self.uri.clone();

        trace!("Start subscriber thread for event");

        thread::spawn(move || {
            trace!("Subscribe");

            let fut = TcpStream::connect(&_uri)
                .and_then(|stream| {
                    // connect() returns a future of an AMQP Client
                    // that resolves once the handshake is done
                    Client::connect(stream, ConnectionOptions::default())
                })
                .and_then(|(client, heartbeat)| {
                    // The heartbeat future should be run in a dedicated thread so that nothing can prevent it from
                    // dispatching events on time.
                    // If we ran it as part of the "main" chain of futures, we might end up not sending
                    // some heartbeats if we don't poll often enough (because of some blocking task or such).
                    tokio::spawn(heartbeat.map_err(|_| ()));

                    // create_channel returns a future that is resolved
                    // once the channel is successfully created
                    client.create_channel()
                })
                .and_then(move |channel| {
                    info!("created channel");

                    channel
                        .queue_declare(
                            "queue_name_here",
                            QueueDeclareOptions::default(),
                            FieldTable::new(),
                        )
                        .map(|queue| (channel, queue))
                })
                .and_then(move |(channel, queue)| {
                    trace!("Bind queue");

                    channel
                        .queue_bind(
                            &"queue_name_here",
                            &"exchange_here",
                            &"queue_name_here",
                            QueueBindOptions::default(),
                            FieldTable::new(),
                        )
                        .map(|_| (channel, queue))
                })
                .and_then(move |(channel, queue)| {
                    info!("channel declared queue {}", "queue_name_here");

                    // basic_consume returns a future of a message
                    // stream. Any time a message arrives for this consumer,
                    // the for_each method would be called
                    channel
                        .basic_consume(
                            &queue,
                            "exchange_here",
                            BasicConsumeOptions::default(),
                            FieldTable::new(),
                        )
                        .map(|stream| (channel, stream))
                })
                .and_then(move |(channel, stream)| {
                    info!("got consumer stream");

                    stream.for_each(move |message| {
                        debug!("got message: {:?}", message);

                        let payload =
                            str::from_utf8(&message.data).expect("Message to string failed");
                        trace!("Message payload {}", payload);
                        let data: Event<ED> =
                            serde_json::from_str(payload).expect("Decode message JSON");
                        trace!("Received message with ID {}: {}", data.id, payload);

                        handler(data);

                        channel.basic_ack(message.delivery_tag, false)
                    })
                })
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "Heartbeat spawn error"));

            trace!("Begin listen");

            Runtime::new().unwrap().block_on_all(fut).unwrap();
            // tokio::spawn(fut);

            // let (channel, stream) = Runtime::new()
            //     .unwrap()
            //     .block_on_all(fut)
            //     .expect("Subscriber spawn failed");

            // trace!("Got channel, stream");

            // _store.some_func();

            // stream.for_each(move |message| {
            //     debug!("got message: {:?}", message);

            //     let payload = str::from_utf8(&message.data).expect("Message to string failed");
            //     trace!("Message payload {}", payload);
            //     let data: Event<TestEvent> =
            //         serde_json::from_str(payload).expect("Decode message JSON");
            //     trace!("Received message with ID {}: {}", data.id, payload);

            //     // handler(data, &_store);

            //     channel.basic_ack(message.delivery_tag, false)
            // });
        })
    }
}
