//! AMQP emitter implementation

use adapters::{EmitterAdapter, EventHandler};
use futures::future::{ok, Future};
use futures::Stream;
use lapin::channel::{
    BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin::client::{Client, ConnectionOptions};
use lapin::types::FieldTable;
use serde_json;
use std::error::Error;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::str;
use std::sync::mpsc;
use tokio;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use Event;
use EventData;

/// AMQP emitter
pub struct AMQPEmitterAdapter<E>
where
    E: EventData,
{
    phantom: PhantomData<E>,
    channel: Channel<TcpStream>,
    exchange: String,
    namespace: String,
    runtime: Runtime,
}

impl<E> AMQPEmitterAdapter<E>
where
    E: EventData,
{
    /// Create a new AMQPEmiterAdapter
    pub fn new(uri: SocketAddr, _exchange: String, namespace: String) -> Self {
        let (tx, rx) = mpsc::channel();
        let exchange = _exchange.clone();
        let mut runtime = Runtime::new().unwrap();
        runtime
            .block_on(
                TcpStream::connect(&uri)
                    .and_then(|stream| Client::connect(stream, ConnectionOptions::default()))
                    .and_then(|(client, heartbeat)| {
                        tokio::spawn(heartbeat.map_err(|_| ()));
                        client.create_channel()
                    }).and_then(move |channel: Channel<TcpStream>| {
                        let ch = channel.clone();
                        tx.send(ch).unwrap();
                        channel.exchange_declare(
                            &exchange,
                            &"topic",
                            ExchangeDeclareOptions {
                                durable: true,
                                ..ExchangeDeclareOptions::default()
                            },
                            FieldTable::new(),
                        )
                    }),
            ).unwrap();
        let channel = rx.recv().unwrap();
        Self {
            phantom: PhantomData,
            channel,
            namespace,
            exchange: _exchange,
            runtime,
        }
    }
}

fn publish<'a>(
    channel: Channel<TcpStream>,
    exchange: String,
    routing_key: String,
    payload: Vec<u8>,
) -> impl Future<Item = (), Error = ()> {
    channel
        .basic_publish(
            &exchange,
            &routing_key,
            payload,
            BasicPublishOptions::default(),
            BasicProperties::default(),
        ).and_then(|_| ok(()))
        .map_err(|_| ())
}

fn prepare_subscription<'a, E>(
    queue_name: String,
    event_name: String,
    exchange: String,
    handler: Box<EventHandler<E>>,
    channel: Channel<TcpStream>,
) -> impl Future<Item = (), Error = Box<Error + 'a>>
where
    E: EventData,
{
    let c_channel = channel.clone();
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
                    stream.for_each(move |message| {
                        let data: Event<E> =
                            serde_json::from_str(str::from_utf8(&message.data).unwrap()).unwrap();
                        handler(&data);
                        c_channel.basic_ack(message.delivery_tag, false)
                    })
                })
        }).and_then(|_| ok(()))
        .map_err(|e| e.into())
}

impl<E> EmitterAdapter<E> for AMQPEmitterAdapter<E>
where
    E: EventData + Send + 'static,
{
    fn emit(&mut self, event: &Event<E>) {
        let payload: Vec<u8> = serde_json::to_string(event).unwrap().into();
        let event_type = event.data().event_type();
        self.runtime.spawn(publish(
            self.channel.clone(),
            self.exchange.clone(),
            event_type,
            payload,
        ));
        // TODO I need the event name here. We may need to rethink the event structure
    }

    fn subscribe(&mut self, event_name: String, handler: Box<EventHandler<E>>) {
        let queue_name = format!("{}-{}", &self.namespace, &event_name);

        self.runtime.spawn(
            prepare_subscription(
                queue_name.clone(),
                event_name,
                self.exchange.clone(),
                handler,
                self.channel.clone(),
            ).map_err(move |e| {
                error!(
                    "Something failed in the {} subscription: {:?}",
                    queue_name, e
                );
            }),
        );
    }
}
