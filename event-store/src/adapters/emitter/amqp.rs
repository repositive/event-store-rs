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
use tokio;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use utils::BoxedFuture;

/// AMQP emitter options
#[derive(Clone)]
pub struct AMQPEmitterOptions {
    /// The name of the exchange to use
    pub exchange: String,

    /// AMQP connection URI
    pub uri: SocketAddr,

    /// Namespace to operate under
    ///
    /// This should be the namespace of the domain or application the event store is running under.
    /// For example, `accounts` or `admin-tool`. This is semantically different from individual
    /// event namespaces, even if they are likely to have the same value.
    pub namespace: &'static str,
}

impl Default for AMQPEmitterOptions {
    fn default() -> Self {
        Self {
            exchange: "default".into(),
            uri: "127.0.0.1:5672".parse().unwrap(),
            namespace: "default",
        }
    }
}

/// AMQP emitter
#[derive(Clone)]
pub struct AMQPEmitterAdapter {
    options: AMQPEmitterOptions,
    channel: Channel<TcpStream>,
}

impl AMQPEmitterAdapter {
    /// Create a new AMQPEmiterAdapter
    pub fn new(options: AMQPEmitterOptions) -> Self {
        let channel = connect(&options.uri, options.exchange.clone());

        let channel = Runtime::new().unwrap().block_on(channel).unwrap();

        Self { options, channel }
    }
}

fn create_consumer<ED>(
    channel: Channel<TcpStream>,
    // TODO: Deleteme
    // queue_name: &str,
    exchange: String,
    namespace: &'static str,
) -> impl Future<Item = (Channel<TcpStream>, Consumer<TcpStream>), Error = io::Error> + Send + 'static
where
    ED: EventData + 'static,
{
    let event_namespace = ED::event_namespace();
    let event_type = ED::event_type();
    let event_name = format!("{}.{}", event_namespace, event_type);
    let queue_name = format!("{}-{}", namespace, event_name);

    info!(
        "Creating consumer queue {} for event {} on exchange {}",
        queue_name, event_name, exchange
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
                    &queue_name,
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
}

fn connect(
    uri: &SocketAddr,
    exchange: String,
) -> impl Future<Item = Channel<TcpStream>, Error = io::Error> {
    TcpStream::connect(uri)
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

            client.create_channel()
        })
        .and_then(move |channel| {
            trace!("Exchange declare {}", exchange);

            channel
                .exchange_declare(
                    &exchange,
                    &"topic",
                    ExchangeDeclareOptions {
                        durable: true,
                        ..ExchangeDeclareOptions::default()
                    },
                    FieldTable::new(),
                )
                .map(|_| channel)
        })
}

impl EmitterAdapter for AMQPEmitterAdapter {
    fn emit<ED: EventData>(&self, event: &Event<ED>) -> BoxedFuture<(), io::Error> {
        let event_namespace = ED::event_namespace();
        let event_type = ED::event_type();

        self.emit_with_string_ident(event_namespace, event_type, &to_value(event).unwrap())
    }

    fn emit_with_string_ident(
        &self,
        event_namespace: &str,
        event_type: &str,
        event: &JsonValue,
    ) -> BoxedFuture<(), io::Error> {
        let payload: Vec<u8> = serde_json::to_string(event)
            .expect("Cant serialise event")
            .into();

        let event_name = format!("{}.{}", event_namespace, event_type);
        let queue_name = format!("{}-{}", self.options.namespace, event_name);

        // TODO: Fix all these clones
        let _channel = self.channel.clone();
        let _options = self.options.clone();
        let _options2 = _options.clone();

        trace!(
            "Emit event {} (ID {}) to queue {} on exchange {}",
            event_name,
            event["id"],
            queue_name,
            self.options.exchange,
        );

        // TODO: Stop connecting all the time
        // let fut = connect(&self.options.uri, self.options.exchange.clone())
        //     .and_then(move |channel| {
        //         channel
        //             .queue_declare(
        //                 &queue_name,
        //                 QueueDeclareOptions {
        //                     durable: true,
        //                     exclusive: false,
        //                     auto_delete: false,
        //                     ..QueueDeclareOptions::default()
        //                 },
        //                 FieldTable::new(),
        //             )
        //             .map(|_| {
        //                 trace!("Queue declared");
        //                 (channel, queue_name)
        //             })
        //     })

        let fut = _channel
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
            .map(|_| (_channel, queue_name))
            .and_then(move |(channel, queue_name)| {
                trace!("Bind to queue {}", queue_name);

                channel
                    .queue_bind(
                        &queue_name,
                        &_options.exchange,
                        &event_name,
                        QueueBindOptions::default(),
                        FieldTable::new(),
                    )
                    .map(|_| (channel, _options.exchange, event_name))
            })
            .and_then(move |(channel, exchange, event_name)| {
                trace!("Basic publish {} to exchange {}", event_name, exchange);

                channel
                    .basic_publish(
                        &exchange,
                        &event_name,
                        payload,
                        BasicPublishOptions::default(),
                        BasicProperties::default(),
                    )
                    .map(move |_| {
                        trace!("Published {}", event_name);

                        channel
                    })
            })
            .map(|_| ());

        Box::new(fut)
    }

    fn subscribe<ED, H>(&self, handler: H) -> BoxedFuture<(), io::Error>
    where
        ED: EventData + 'static,
        H: Fn(Event<ED>) -> () + Send + 'static,
    {
        let _exchange = self.options.exchange.clone();
        let _namespace = self.options.namespace;

        // Box::new(FutOk(()))

        let consumer = connect(&self.options.uri, _exchange.clone())
            .and_then(move |channel| create_consumer::<ED>(channel, _exchange, _namespace))
            .and_then(move |(channel, stream)| {
                trace!("Begin consuming stream");

                let consume = stream
                    .for_each(move |message| {
                        let payload = str::from_utf8(&message.data).unwrap();
                        let data: Event<ED> = serde_json::from_str(payload).unwrap();
                        trace!("Received message with ID {}", data.id);
                        handler(data);
                        trace!("Ack {}", message.delivery_tag);
                        channel.basic_ack(message.delivery_tag, false)
                    })
                    .map_err(|e| panic!("Consumer stream panicked: {}", e));

                tokio::spawn(consume);

                FutOk(())
            });

        Box::new(consumer)
    }
}
