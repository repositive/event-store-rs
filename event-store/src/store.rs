use crate::aggregator::Aggregator;
use crate::amqp::*;
use crate::event::Event;
use crate::pg::*;
use crate::store_query::StoreQuery;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use futures::Future;
use lapin_futures::channel::Channel;
use log::{debug, trace};
use r2d2::Pool;
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use std::fmt;
use std::fmt::Debug;
use std::io;
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
        channel: Channel<TcpStream>,
    ) -> Self {
        Self {
            store_namespace,
            pool,
            channel,
        }
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

        let queue_name = self.namespaced_event_queue_name::<ED>();

        let _channel = self.channel.clone();

        pg_save(self.pool.get().unwrap(), event)
            .and_then(|event| amqp_emit_event(_channel, queue_name, "test_exchange".into(), event))
            .map(|_| ())
    }

    pub fn last_event<ED>(&self) -> impl Future<Item = Option<Event<ED>>, Error = io::Error>
    where
        ED: EventData,
    {
        pg_last_event::<ED>(self.pool.get().unwrap())
    }

    fn namespaced_event_queue_name<ED>(&self) -> String
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
