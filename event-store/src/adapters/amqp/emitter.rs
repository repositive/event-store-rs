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
use tokio;
use tokio::net::TcpStream;
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
        exchange: String,
        namespace: String,
    ) -> Box<Future<Item = Self, Error = io::Error> + Send> {
        let exchange1 = exchange.clone();
        info!("Connecting to AMQP using {}", uri);
        Box::new(
            TcpStream::connect(&uri)
                .and_then(|stream| Client::connect(stream, ConnectionOptions::default()))
                .and_then(|(client, heartbeat)| {
                    info!("Connection stablished");
                    info!("Starting heartbeat task");
                    tokio::spawn(heartbeat.map_err(|_| ()));
                    info!("Creating amqp channel");
                    client.create_channel()
                })
                .and_then(move |channel: Channel<TcpStream>| {
                    let ch = channel.clone();
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
                        .and_then(move |_| ok(ch))
                })
                .and_then(|channel| {
                    ok(Self {
                        channel,
                        namespace,
                        exchange,
                    })
                }),
        )
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
    H: Fn(&Event<E>) -> () + Send + 'static,
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
                    let handle_events = stream
                        .for_each(move |message| {
                            let data: Event<E> = serde_json::from_str(
                                str::from_utf8(&message.data).unwrap(),
                            ).unwrap();
                            info!("Receiving message with id {}", data.id);
                            handler(&data);
                            c_channel.basic_ack(message.delivery_tag, false)
                        })
                        .map_err(|e| {
                            panic!(e);
                        });

                    info!("Starting to consume from queue {}", queue_name1);
                    tokio::spawn(handle_events);
                    ok(())
                })
        })
        .and_then(|_| ok(()))
        .map_err(|e| e.into())
}

impl EmitterAdapter for AMQPEmitterAdapter {
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

    fn subscribe<ED, H>(&self, handler: H) -> Box<Future<Item = (), Error = io::Error> + Send>
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
}
