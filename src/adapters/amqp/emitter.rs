//! AMQP emitter implementation

use adapters::{EmitterAdapter, EventHandler};
use futures::Future;
use futures::Stream;
use lapin::channel::{
    BasicConsumeOptions, Channel, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
};
use lapin::client::{Client, ConnectionOptions};
use lapin::types::FieldTable;
use serde_json;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::str;
use std::sync::mpsc;
use tokio;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;

use Events;

/// AMQP emitter
pub struct AMQPEmitterAdapter<E>
where
    E: Events,
{
    phantom: PhantomData<E>,
    subscribers: HashMap<String, EventHandler<E>>,
    channel: Channel<TcpStream>,
    exchange: String,
    namespace: String,
    runtime: Runtime,
}

impl<E> AMQPEmitterAdapter<E>
where
    E: Events,
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
                    })
                    .and_then(move |channel: Channel<TcpStream>| {
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
            )
            .unwrap();
        let channel = rx.recv().unwrap();
        Self {
            phantom: PhantomData,
            subscribers: HashMap::new(),
            channel,
            namespace,
            exchange: _exchange,
            runtime,
        }
    }
}

impl<E> EmitterAdapter<E> for AMQPEmitterAdapter<E>
where
    E: Events + Send,
{
    fn get_subscriptions(&self) -> &HashMap<String, EventHandler<E>> {
        &self.subscribers
    }

    fn emit(&self, _event: &E) {
        // TODO I need the event name here. We may need to rethink the event structure
    }

    fn subscribe(&mut self, event_name: String, handler: EventHandler<E>) {
        let queue_name = format!("{}-{}", &self.namespace, &event_name);
        let channel = self.channel.clone();
        let to_run = self
            .channel
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
            .and_then(|queue| {
                self.channel
                    .queue_bind(
                        &queue_name,
                        &self.exchange,
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
            })
            .and_then(|stream| {
                stream.for_each(|message| {
                    let data: E =
                        serde_json::from_str(str::from_utf8(&message.data).unwrap()).unwrap();
                    handler(&data);
                    self.channel.basic_ack(message.delivery_tag, false)
                })
            });

        // self.runtime.spawn(to_run.into());
    }

    fn unsubscribe(&mut self, _event_name: String) {
        &self.subscribers.remove(&_event_name);
    }
}
