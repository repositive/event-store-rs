use crate::aggregator::Aggregator;
use crate::amqp::*;
use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::pg::*;
use crate::store_query::StoreQuery;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use futures::{future, Future};
use lapin_futures::channel::Channel;
use log::{error, trace};
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

        // TODO: Subscribe to event replay requested events

        // TODO: Pass in an AMQP adapter inside a promise instead of doing this here
        amqp_connect(addr, "test_exchange".into()).map(|channel| Self {
            channel,
            store_namespace,
            pool,
        })
    }

    pub fn aggregate<T, QA, E>(&self, query_args: QA) -> impl Future<Item = T, Error = io::Error>
    where
        E: Events,
        T: Aggregator<E, QA, PgQuery>,
        QA: Clone,
    {
        let store_query = T::query(query_args.clone());
        let cache_key = store_query.unique_id();

        pg_cache_read(self.pool.get().unwrap(), cache_key)
            .and_then(
                |(conn, cache_result): (
                    PooledConnection<PostgresConnectionManager>,
                    Option<CacheResult<T>>,
                )| {
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
        ED: EventData,
    {
        // TODO: Dedupe this
        let queue_name = format!(
            "{}-{}.{}",
            self.store_namespace,
            ED::event_namespace(),
            ED::event_type()
        );

        let _channel = self.channel.clone();

        pg_save(self.pool.get().unwrap(), event)
            .and_then(|event| amqp_emit_event(_channel, queue_name, "test_exchange".into(), event))
            .map(|_| ())
    }

    pub fn subscribe<ED>(&self) -> impl Future<Item = (), Error = io::Error>
    where
        ED: EventHandler + 'static,
    {
        // TODO: Dedupe this
        let queue_name = format!(
            "{}-{}.{}",
            self.store_namespace,
            ED::event_namespace(),
            ED::event_type()
        );

        let _self = self.clone();

        let consumer = amqp_create_consumer(
            self.channel.clone(),
            queue_name,
            "test_exchange".into(),
            move |event: Event<ED>| {
                // TODO: Save event in this closure somewhere

                ED::handle_event(event, &_self);
            },
        );

        tokio::spawn(consumer.map_err(|e| {
            error!("Consumer error: {}", e);

            ()
        }));

        // TODO: Send an event replay requested event

        future::ok(())
    }
}
