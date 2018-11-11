use crate::{Event, Store};
use futures::future::{ok as FutOk, Future};
use futures::stream::Stream;
use lapin_futures::channel::{
    BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin_futures::client::{Client, ConnectionOptions};
use lapin_futures::types::FieldTable;
use std::fmt;
use std::net::SocketAddr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct AMQPEmitterAdapter {
    sender: AMQPSender,
    receiver: AMQPReceiver,
}

impl AMQPEmitterAdapter {
    pub fn new(uri: SocketAddr, exchange: String) -> Self {
        Self {
            sender: AMQPSender::new(uri, exchange.clone()),
            receiver: AMQPReceiver::new(uri, exchange.clone()),
        }
    }

    pub fn split(self) -> (AMQPSender, AMQPReceiver) {
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
    pub fn new(uri: SocketAddr, exchange: String) -> Self {
        let channel = Runtime::new()
            .unwrap()
            .block_on(
                TcpStream::connect(&uri)
                    .and_then(|stream| {
                        trace!("Sender stream connected");

                        Client::connect(stream, ConnectionOptions::default())
                    })
                    .and_then(|(client, heartbeat)| {
                        tokio::spawn(heartbeat.map_err(|_| ()));

                        trace!("Heartbeat spawned");

                        // create_channel returns a future that is resolved
                        // once the channel is successfully created
                        client.create_channel()
                    })
                    .and_then(move |channel| {
                        trace!("Exchange declare");

                        channel
                            .exchange_declare(
                                &"exchange_here",
                                &"topic",
                                ExchangeDeclareOptions {
                                    durable: true,
                                    auto_delete: false,
                                    ..ExchangeDeclareOptions::default()
                                },
                                FieldTable::new(),
                            )
                            .map(|_| channel)
                    })
                    .and_then(move |channel| {
                        trace!("Channel declared");

                        FutOk(channel)
                    }),
            )
            .expect("runtime failure");

        trace!("Channel created");

        Self {
            uri,
            exchange,
            channel,
        }
    }

    pub fn emit(&self, event: Event) {
        Runtime::new()
            .unwrap()
            .block_on_all(
                self.channel.basic_publish(
                    "exchange_here",
                    "queue_name_here",
                    format!("Emit event, number: {:?}", event)
                        .as_bytes()
                        .to_vec(),
                    BasicPublishOptions::default(),
                    BasicProperties::default(),
                ),
            )
            .expect("Could not emit");
    }
}

#[derive(Debug)]
pub struct AMQPReceiver {
    uri: SocketAddr,
    exchange: String,
    // tx: Sender<Event>,
    // rx: Receiver<Event>,
}

impl AMQPReceiver {
    pub fn new(uri: SocketAddr, exchange: String) -> Self {
        Self {
            uri,
            exchange,
            // tx,
            // rx,
        }
    }

    pub fn subscribe<H>(&self, store: Store, handler: H) -> JoinHandle<()>
    where
        H: Fn(Event, &Store) -> () + Send + 'static,
    {
        let _uri = self.uri.clone();
        // let _store = store.clone();
        let (tx, rx) = channel();

        trace!("Start subscriber thread for event");

        let handle = thread::spawn(move || {
            trace!("Subscribe");

            // stream.for_each(move |message| {
            //                     debug!("got message: {:?}", message);
            //                     info!(
            //                         "decoded message: {:?}",
            //                         std::str::from_utf8(&message.data).unwrap()
            //                     );

            //                     // TODO: Actual event payload
            //                     handler(321, &store);

            //                     channel.basic_ack(message.delivery_tag, false)
            //                 })

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
                        info!(
                            "decoded message: {:?}",
                            std::str::from_utf8(&message.data).unwrap()
                        );

                        // TODO: Actual event payload
                        // handler(321, &_store);
                        tx.send(321).expect("Failed to send");

                        trace!("TXed");

                        // TODO: This is still acked even if the handler fails!
                        channel.basic_ack(message.delivery_tag, false)
                    })
                });

            //  let (channel, stream) = Runtime::new()
            //      .unwrap()
            //      .block_on_all(fut)
            //      .expect("Subscribe runtime failure");

            // ;

            Runtime::new()
                .unwrap()
                .block_on_all(fut)
                .expect("Subscriber spawn failed");
        });

        for msg in rx {
            trace!("RX: {}", msg);

            handler(msg, &store);
        }

        handle
    }
}
