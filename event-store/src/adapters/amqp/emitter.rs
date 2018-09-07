//! AMQP emitter implementation

use adapters::EmitterAdapter;
use futures::future::{ok as FutOk, Future};
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
use utils::BoxedFuture;
use Event;
use EventData;

/// AMQP emitter
#[derive(Clone)]
pub struct AMQPEmitterAdapter {
    channel: Channel<TcpStream>,
    exchange: String,
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
                    info!("Connection stablished");
                    info!("Starting heartbeat task");
                    tokio::spawn(heartbeat.map_err(|_| ()));
                    info!("Creating amqp channel");
                    client.create_channel()
                }).and_then(move |channel: Channel<TcpStream>| {
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
                        ).and_then(move |_| FutOk(ch))
                }).and_then(|channel| FutOk(Self { channel, exchange })),
        )
    }
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
    let queue_name = format!("{}-{}", event_namespace, event_name);
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
        ).and_then(move |queue| {
            info!("Binding queue {} to exchange {}", queue_name, exchange);
            channel
                .queue_bind(
                    &queue_name,
                    &exchange,
                    &event_name,
                    QueueBindOptions::default(),
                    FieldTable::new(),
                ).and_then(move |_| {
                    channel.basic_consume(
                        &queue,
                        &queue_name,
                        BasicConsumeOptions::default(),
                        FieldTable::new(),
                    )
                }).and_then(move |stream| {
                    let handle_events = stream
                        .for_each(move |message| {
                            let data: Event<E> =
                                serde_json::from_str(str::from_utf8(&message.data).unwrap())
                                    .unwrap();
                            info!("Receiving message with id {}", data.id);
                            handler(&data);
                            c_channel.basic_ack(message.delivery_tag, false)
                        }).map_err(|e| {
                            panic!(e);
                        });

                    info!("Starting to consume from queue {}", queue_name1);
                    tokio::spawn(handle_events);
                    FutOk(())
                })
        }).and_then(|_| FutOk(()))
        .map_err(|e| e.into())
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
                ).and_then(move |_| {
                    info!("Event with id {} delivered", id);
                    FutOk(())
                }),
        )
    }

    fn subscribe<'a, ED, H>(&self, handler: H) -> BoxedFuture<'a, (), io::Error>
    where
        ED: EventData + 'a,
        H: Fn(&Event<ED>) -> () + Send + 'static,
    {
        Box::new(prepare_subscription(
            self.exchange.clone(),
            handler,
            self.channel.clone(),
        ))
    }
}
