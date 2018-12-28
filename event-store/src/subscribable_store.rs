use crate::aggregator::Aggregator;
use crate::amqp::*;
use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::event_replay::EventReplayRequested;
use crate::pg::PgQuery;
use crate::store::Store;
use chrono::naive::NaiveDateTime;
use chrono::prelude::*;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use lapin_futures::channel::Channel;
use lapin_futures::consumer::Consumer;
use log::{debug, info, trace};
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use std::fmt;
use std::fmt::Debug;
use std::io;
use std::net::SocketAddr;
use tokio::net::tcp::TcpStream;
use tokio::prelude::*;

#[derive(Debug, Clone)]
pub struct SubscribeOptions {
    replay_previous_events: bool,
    save_on_receive: bool,
}

impl Default for SubscribeOptions {
    fn default() -> Self {
        Self {
            replay_previous_events: true,
            save_on_receive: true,
        }
    }
}

#[derive(Clone)]
pub struct SubscribableStore {
    store_namespace: String,
    channel: Channel<TcpStream>,
    inner_store: Store,
}

impl Debug for SubscribableStore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SubscribableStore namespace {}", self.store_namespace)
    }
}

impl SubscribableStore {
    pub async fn new(
        store_namespace: String,
        pool: Pool<PostgresConnectionManager>,
    ) -> Result<Self, io::Error> {
        let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();

        // TODO: Pass in an AMQP adapter inside a promise instead of doing this here
        let channel = await!(amqp_connect(addr, "test_exchange".into()))?;

        let store = Self {
            channel: channel.clone(),
            store_namespace: store_namespace.clone(),
            inner_store: Store::new(store_namespace, pool, channel),
        };

        await!(store.subscribe::<EventReplayRequested>(SubscribeOptions {
            replay_previous_events: false,
            save_on_receive: false
        }))?;

        Ok(store)
    }

    pub async fn aggregate<'a, T, QA, E>(&'a self, query_args: &'a QA) -> Result<T, io::Error>
    where
        E: Events,
        T: Aggregator<E, QA, PgQuery>,
        QA: Clone + Debug + 'a,
    {
        let res: T = await!(self.inner_store.aggregate::<'a, T, QA, E>(&query_args))?;

        Ok(res)
    }

    pub async fn save<'a, ED>(&'a self, event: &'a Event<ED>) -> Result<(), io::Error>
    where
        ED: EventData + Debug + Send + Sync,
    {
        await!(self.inner_store.save(event))
    }

    pub async fn save_no_emit<'a, ED>(&'a self, event: &'a Event<ED>) -> Result<(), io::Error>
    where
        ED: EventData + Debug + Send + Sync,
    {
        await!(self.inner_store.save_no_emit(event))
    }

    pub async fn subscribe<ED>(&self, options: SubscribeOptions) -> Result<(), io::Error>
    where
        ED: EventHandler + Debug + Send + Sync + 'static,
    {
        let replay_queue_name = self.event_queue_name::<EventReplayRequested>();

        info!(
            "Starting subscription to {}",
            ED::event_namespace_and_type()
        );

        let inner_channel = self.channel.clone();
        let queue_name = self.namespaced_event_queue_name::<ED>();

        debug!("Begin listening for events on queue {}", queue_name);

        let channel = self.channel.clone();
        let inner_store = self.inner_store.clone();
        let inner_options = options.clone();

        tokio::spawn_async(
            async {
                let ch = channel.clone();
                let mut stream: Consumer<TcpStream> = await!(amqp_create_consumer::<ED>(
                    channel,
                    queue_name,
                    "test_exchange".into(),
                ))
                .expect("Subscribe failed");

                trace!("Before while loop");

                // Oh my dog Rust why
                let inner_inner_store = inner_store;
                let inner_inner_options = inner_options;

                while let Some(Ok(message)) = await!(stream.next()) {
                    let payload = std::str::from_utf8(&message.data).unwrap();
                    let event: Event<ED> = serde_json::from_str(payload).unwrap();

                    trace!("Received event {:?}", event);

                    let saved = if inner_inner_options.save_on_receive {
                        await!(inner_inner_store.save_no_emit(&event))
                    } else {
                        Ok(())
                    };

                    saved
                        .map(|_| {
                            ED::handle_event(event, &inner_inner_store);

                            ch.basic_ack(message.delivery_tag, false);
                        })
                        .expect("Could not save event");
                }
            },
        );

        if options.replay_previous_events {
            trace!("Subscription started, emitting replay request");

            let since = await!(self.inner_store.last_event::<ED>())
                .map(|last_event| {
                    trace!("Fetched last event {:?}", last_event);

                    last_event.map(|ev| ev.context.time).unwrap_or_else(|| {
                        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
                    })
                })
                .unwrap_or_else(|_| {
                    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
                });

            let replay_event = EventReplayRequested::from_event::<ED>(since);

            await!(amqp_emit_event(
                inner_channel,
                replay_queue_name,
                "test_exchange".into(),
                &replay_event,
            ))?;

            trace!("Replay request emitted");
        }

        Ok(())
    }

    fn namespaced_event_queue_name<ED>(&self) -> String
    where
        ED: EventData,
    {
        format!("{}-{}", self.store_namespace, self.event_queue_name::<ED>())
    }

    fn event_queue_name<ED>(&self) -> String
    where
        ED: EventData,
    {
        format!("{}.{}", ED::event_namespace(), ED::event_type())
    }
}
