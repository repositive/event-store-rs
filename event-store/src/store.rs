use crate::aggregator::Aggregator;
use crate::amqp::*;
use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::event_replay::EventReplayRequested;
use crate::pg::*;
use crate::store_query::StoreQuery;
use chrono::naive::NaiveDateTime;
use chrono::prelude::*;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use futures::Future;
use lapin_futures::channel::Channel;
use log::{debug, error, trace};
use r2d2::Pool;
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use std::fmt;
use std::fmt::Debug;
use std::io;
use std::net::SocketAddr;
use tokio::net::tcp::TcpStream;

#[derive(Clone)]
pub struct Store {
    store_namespace: String,
    channel: Channel<TcpStream>,
    pool: Pool<PostgresConnectionManager>,
}

impl Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Store namespace {}", self.store_namespace)
    }
}

impl Store {
    pub fn new(
        store_namespace: String,
        pool: Pool<PostgresConnectionManager>,
    ) -> impl Future<Item = Self, Error = io::Error> {
        let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();

        // TODO: Pass in an AMQP adapter inside a promise instead of doing this here
        amqp_connect(addr, "test_exchange".into())
            .map(|channel| Self {
                channel,
                store_namespace,
                pool,
            })
            .and_then(|store| {
                debug!("Begin listening for event replay requests");

                store.subscribe::<EventReplayRequested>().map(|_| store)
            })
    }

    pub fn aggregate<T, QA, E>(&self, query_args: QA) -> impl Future<Item = T, Error = io::Error>
    where
        E: Events,
        T: Aggregator<E, QA, PgQuery>,
        QA: Clone + Debug,
    {
        debug!("Aggregate with arguments {:?}", query_args);

        let store_query = T::query(query_args.clone());
        let cache_key = store_query.unique_id();
        let debug_cache_key = cache_key.clone();

        pg_cache_read(self.pool.get().unwrap(), cache_key)
            .and_then(
                move |(conn, cache_result): (
                    PooledConnection<PostgresConnectionManager>,
                    Option<CacheResult<T>>,
                )| {
                    trace!(
                        "Aggregate cache key {} result {:?}",
                        debug_cache_key,
                        cache_result
                    );

                    let (initial_state, since) = cache_result
                        .map(|res| (res.0, Some(res.1)))
                        .unwrap_or_else(|| (T::default(), None));

                    trace!(
                        "Aggregate initial state {:?}, since {:?}",
                        initial_state,
                        since
                    );

                    pg_read(conn, store_query, since).map(|events: Vec<E>| (events, initial_state))
                },
            )
            .map(|(events, initial_state)| {
                trace!("Read {} events to aggregate", events.len());

                events.iter().fold(initial_state, T::apply_event)
            })
    }

    pub fn save<ED>(&self, event: Event<ED>) -> impl Future<Item = (), Error = io::Error>
    where
        ED: EventData + Debug,
    {
        debug!("Save event {:?}", event);

        let queue_name = self.event_queue_name::<ED>();

        let _channel = self.channel.clone();

        pg_save(self.pool.get().unwrap(), event)
            .and_then(|event| amqp_emit_event(_channel, queue_name, "test_exchange".into(), event))
            .map(|_| ())
    }

    pub fn subscribe<ED>(&self) -> impl Future<Item = (), Error = io::Error>
    where
        ED: EventHandler + Debug + 'static,
    {
        let queue_name = self.event_queue_name::<ED>();
        let queue_name_2 = queue_name.clone();

        let inner_self = self.clone();
        let inner_channel = self.channel.clone();
        let inner_channel_2 = inner_channel.clone();

        debug!("Begin listening for events on queue {}", queue_name);

        let consumer = amqp_create_consumer(
            inner_channel,
            queue_name,
            "test_exchange".into(),
            move |event: Event<ED>| {
                // TODO: Save event in this closure somewhere

                ED::handle_event(event, &inner_self);
            },
        );

        tokio::spawn(consumer.map_err(|e| {
            error!("Consumer error: {}", e);

            ()
        }));

        pg_last_event::<ED>(self.pool.get().unwrap())
            .map(|last_event| {
                trace!("Fetched last event {:?}", last_event);

                last_event.map(|ev| ev.context.time).unwrap_or_else(|| {
                    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
                })
            })
            .and_then(move |since| {
                trace!("Emit replay request for events since {:?}", since);

                amqp_emit_event(
                    inner_channel_2,
                    queue_name_2,
                    "test_exchange".into(),
                    EventReplayRequested::from_event::<ED>(since),
                )
            })
            .map(|_| ())
    }

    fn event_queue_name<ED>(&self) -> String
    where
        ED: EventData,
    {
        format!(
            "{}-{}.{}",
            self.store_namespace,
            ED::event_namespace(),
            ED::event_type()
        )
    }
}
