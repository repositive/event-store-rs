use crate::event::Event;
use crate::forward;
use event_store_derive_internals::EventData;
use futures::Future;
use lapin_futures::channel::{
    BasicConsumeOptions, BasicProperties, BasicPublishOptions, Channel, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin_futures::client::{Client, ConnectionOptions};
use lapin_futures::consumer::Consumer;
use lapin_futures::queue::Queue;
use lapin_futures::types::FieldTable;
use log::{debug, info};
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;

/// Connect to AMQP
pub async fn amqp_connect(
    uri: SocketAddr,
    exchange: String,
) -> Result<Channel<TcpStream>, io::Error> {
    let exchange1 = exchange.clone();

    let stream: TcpStream = await!(forward(TcpStream::connect(&uri)))?;

    let (client, heartbeat) = await!(forward(Client::connect(
        stream,
        ConnectionOptions::default()
    )))
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    // .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())));

    tokio::spawn(heartbeat.map_err(|e| eprintln!("heartbeat error: {:?}", e)));

    let channel = await!(forward(client.create_channel()))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    await!(forward(channel.exchange_declare(
        &exchange1,
        &"topic",
        ExchangeDeclareOptions {
            durable: true,
            ..ExchangeDeclareOptions::default()
        },
        FieldTable::new(),
    )))
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok(channel)

    // TcpStream::connect(&uri)
    //     .and_then(|stream| {
    //         Client::connect(stream, ConnectionOptions::default())
    //             .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    //     })
    //     .and_then(|(client, heartbeat)| {
    //         trace!("Start heartbeat");

    //         tokio::spawn(heartbeat.map_err(|e| eprintln!("heartbeat error: {:?}", e)))
    //             .into_future()
    //             .map(|_| client)
    //             .map_err(|_| io::Error::new(io::ErrorKind::Other, "spawn error"))
    //     })
    //     .and_then(move |client| {
    //         trace!("Set up channel");

    //         client
    //             .create_channel()
    //             .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    //     })
    //     .and_then(move |channel| {
    //         trace!("Declare exchange {}", exchange1);

    //         channel
    //             .exchange_declare(
    //                 &exchange1,
    //                 &"topic",
    //                 ExchangeDeclareOptions {
    //                     durable: true,
    //                     ..ExchangeDeclareOptions::default()
    //                 },
    //                 FieldTable::new(),
    //             )
    //             .map(|_| channel)
    //             .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    //     })
}

/// Create a consumer for an AMQP queue
pub async fn amqp_create_consumer<ED>(
    channel: Channel<TcpStream>,
    queue_name: String,
    exchange: String,
    // handler: H,
) -> Result<Consumer<TcpStream>, io::Error>
where
    ED: EventData,
    // H: Fn(Event<E>) -> () + Unpin,
{
    let event_namespace = ED::event_namespace();
    let event_type = ED::event_type();
    let event_name = format!("{}.{}", event_namespace, event_type);

    info!(
        "Creating consumer for event {} on queue {} on exchange {}",
        event_name, queue_name, exchange
    );

    let queue = await!(amqp_bind_queue(
        &channel,
        &queue_name,
        &exchange,
        &event_name
    ))?;

    let consumer_tag = format!("consumer-{}-{}", exchange, event_name);

    info!(
        "Create consumer on exchange {} with consumer tag {}",
        exchange, consumer_tag
    );

    let stream = await!(forward(
        channel
            .basic_consume(
                &queue,
                &consumer_tag,
                BasicConsumeOptions::default(),
                FieldTable::new(),
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string())),
    ))?;

    info!("Got stream for consumer");

    Ok(stream)

    // stream
    //     .for_each(move |message| {
    //         let payload = str::from_utf8(&message.data).unwrap();
    //         let data: Event<E> = serde_json::from_str(payload).unwrap();

    //         trace!("Received message with ID {}: {}", data.id, payload);

    //         handler(data);

    //         channel.basic_ack(message.delivery_tag, false)
    //     })
    //     .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()));
}

/// Declare and bind an AMQP queue to an exchange
// TODO: Pass in/out options struct instead of magic strings
pub async fn amqp_bind_queue<'a>(
    channel: &'a Channel<TcpStream>,
    queue_name: &'a String,
    exchange_name: &'a String,
    routing_key: &'a String,
    // ) -> impl Future<Item = (Channel<TcpStream>, Queue, String, String, String), Error = io::Error> {
) -> Result<Queue, io::Error> {
    let queue = await!(forward(channel.queue_declare(
        &queue_name,
        QueueDeclareOptions {
            durable: true,
            exclusive: false,
            auto_delete: false,
            ..QueueDeclareOptions::default()
        },
        FieldTable::new(),
    )))
    .unwrap();

    await!(forward(
        channel
            .queue_bind(
                &queue_name,
                &exchange_name,
                &routing_key,
                QueueBindOptions::default(),
                FieldTable::new(),
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    ))?;

    Ok(queue)

    // .map(|queue| (queue, channel))
    // .and_then(move |(queue, channel)| {
    //     debug!(
    //         "Queue {} declared, binding with routing key {}",
    //         queue_name, routing_key
    //     );

    //     channel
    //         .queue_bind(
    //             &queue_name,
    //             &exchange_name,
    //             &routing_key,
    //             QueueBindOptions::default(),
    //             FieldTable::new(),
    //         )
    //         .map(move |_| (channel, queue, queue_name, exchange_name, routing_key))
    // })
    // .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

/// Emit an event onto a queue
pub async fn amqp_emit_event<ED>(
    channel: Channel<TcpStream>,
    queue_name: String,
    exchange: String,
    event: &Event<ED>,
    // ) -> impl Future<Item = (Event<ED>, Channel<TcpStream>), Error = io::Error>
) -> Result<(), io::Error>
where
    ED: EventData,
{
    let payload: Vec<u8> = serde_json::to_string(&event)
        .expect("Cant serialise event")
        .into();

    let event_namespace = ED::event_namespace();
    let event_type = ED::event_type();
    let event_name = format!("{}.{}", event_namespace, event_type);

    info!(
        "Emitting event {} onto exchange {} through queue {}",
        event_name, exchange, queue_name
    );

    await!(amqp_emit_data(
        channel, queue_name, exchange, event_name, payload
    ))?;

    Ok(())
}

/// Emit an event onto a queue
pub async fn amqp_emit_data(
    channel: Channel<TcpStream>,
    queue_name: String,
    exchange: String,
    routing_key: String,
    payload: Vec<u8>,
    // ) -> impl Future<Item = Channel<TcpStream>, Error = io::Error> {
) -> Result<(), io::Error> {
    debug!(
        "Emitting payload through routing key {} onto exchange {}",
        routing_key, exchange
    );

    await!(amqp_bind_queue(
        &channel,
        &queue_name,
        &exchange,
        &routing_key
    ))?;

    await!(forward(
        channel
            .basic_publish(
                &exchange,
                &routing_key,
                payload,
                BasicPublishOptions::default(),
                BasicProperties::default(),
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    ))?;

    Ok(())
}
