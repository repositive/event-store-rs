use crate::aggregator::Aggregator;
use crate::amqp::*;
use crate::event::Event;
use crate::pg::*;
use crate::store_query::StoreQuery;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use lapin_futures::channel::Channel;
use log::{debug, trace};
use r2d2::Pool;
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

    pub async fn aggregate<'a, T, QA, E>(&'a self, query_args: &'a QA) -> Result<T, io::Error>
    where
        E: Events,
        T: Aggregator<E, QA, PgQuery>,
        QA: Clone + Debug + 'a,
    {
        debug!("Aggregate with arguments {:?}", query_args);

        let store_query = T::query(query_args.clone());
        let cache_key = store_query.unique_id();
        let debug_cache_key = cache_key.clone();

        let cache_result = await!(pg_cache_read(self.pool.get().unwrap(), cache_key)).unwrap();

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

        let events = await!(pg_read(self.pool.get().unwrap(), &store_query, since)).unwrap();

        trace!("Read {} events to aggregate", events.len());

        Ok(events.iter().fold(initial_state, T::apply_event))
    }

    pub async fn save<'a, ED>(&'a self, event: &'a Event<ED>) -> Result<(), io::Error>
    where
        ED: EventData + Debug,
    {
        debug!("Save event {:?}", event);

        let queue_name = self.namespaced_event_queue_name::<ED>();

        let _channel = self.channel.clone();

        await!(self.save_no_emit(&event))?;

        await!(amqp_emit_event(
            _channel,
            queue_name,
            "test_exchange".into(),
            event
        ))?;

        Ok(())

        // self.save_no_emit(event)
        //     .and_then(|event| amqp_emit_event(_channel, queue_name, "test_exchange".into(), event))
        //     .map(|(event, _channel)| event)
    }

    pub async fn save_no_emit<'a, ED>(&'a self, event: &'a Event<ED>) -> Result<(), io::Error>
    where
        ED: EventData + Debug,
    {
        debug!("Save event {:?}", event);

        await!(pg_save(self.pool.get().unwrap(), event))?;

        Ok(())
    }

    pub async fn last_event<ED>(&self) -> Result<Option<Event<ED>>, io::Error>
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
