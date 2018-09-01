//! AMQP emitter implementation

use adapters::EmitterAdapter;
use futures::future::{ok, Future};
use futures::Stream;
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
use std::sync::mpsc;
use std::time::Duration;
use tokio;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use Event;
use EventData;
use Events;

/// AMQP emitter
pub struct AMQPEmitterAdapter {
    channel: Channel<TcpStream>,
    exchange: String,
    namespace: String,
}

impl AMQPEmitterAdapter {
    /// Create a new AMQPEmiterAdapter
    pub fn new(
        uri: SocketAddr,
        _exchange: String,
        namespace: String,
        runtime: &mut Runtime,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let exchange = _exchange.clone();
        runtime.spawn(
            TcpStream::connect(&uri)
                .and_then(|stream| Client::connect(stream, ConnectionOptions::default()))
                .and_then(|(client, heartbeat)| {
                    info!("Starting heartbeat task");
                    tokio::spawn(heartbeat.map_err(|_| ()));
                    info!("Creating amqp channel");
                    client.create_channel()
                })
                .and_then(move |channel: Channel<TcpStream>| {
                    let ch = channel.clone();
                    tx.send(ch).expect("Send channel to main thread");
                    channel.exchange_declare(
                        &exchange,
                        &"topic",
                        ExchangeDeclareOptions {
                            durable: true,
                            ..ExchangeDeclareOptions::default()
                        },
                        FieldTable::new(),
                    )
                })
                .map_err(|e| {
                    error!("Error connecting to AMQP: {}", e);
                }),
        );
        let channel = rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| {
                error!("AMQP channel was not made available: {}", e);
                panic!(e);
            })
            .expect("AMQP channel was not made available");
        Self {
            channel,
            namespace,
            exchange: _exchange,
        }
    }
}

fn prepare_subscription<E, H>(
    queue_name: String,
    event_name: String,
    exchange: String,
    handler: H,
    channel: Channel<TcpStream>,
) -> impl Future<Item = (), Error = io::Error>
where
    E: EventData,
    H: Fn(&Event<E>) -> (),
{
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
                    info!("Starting to consume from queue {}", queue_name1);
                    stream.for_each(move |message| {
                        let data: Event<E> =
                            serde_json::from_str(str::from_utf8(&message.data).unwrap()).unwrap();
                        info!("Receiving message with id {}", data.id);
                        handler(&data);
                        c_channel.basic_ack(message.delivery_tag, false)
                    })
                })
        })
        .and_then(|_| ok(()))
        .map_err(|e| e.into())
}

impl EmitterAdapter for AMQPEmitterAdapter {
    fn get_subscriptions(&self) -> Vec<String> {
        vec![]
    }

    fn emit<'a, E: Events + Sync>(
        &self,
        event: &Event<E>,
    ) -> Box<Future<Item = (), Error = io::Error> + Send + Sync> {
        let payload: Vec<u8> = serde_json::to_string(event)
            .expect("Cant serialise event")
            .into();
        let event_type = event.data().event_type();
        let id = event.id;
        info!("Emiting event {} with id {}", event_type, id);

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
                    ok(())
                }),
        )
    }

    fn subscribe<ED, H>(&mut self, handler: H) -> Box<Future<Item = (), Error = io::Error> + Send>
    where
        ED: EventData + 'static,
        H: Fn(&Event<ED>) -> () + Send + 'static,
    {
        let event_name = ED::event_type();
        let queue_name = format!("{}-{}", &self.namespace, &event_name);
        Box::new(prepare_subscription(
            queue_name.clone(),
            event_name.into(),
            self.exchange.clone(),
            handler,
            self.channel.clone(),
        ))
    }

    fn unsubscribe<ED: EventData>(&mut self) {}
}
