//! Adapters for use with event store integrations
//!
//! A store will require a storage backend, cache backend and an event emitter instance for
//! integration with other event-driven domains. Use the adapters exposed by this module to suit
//! your application and architecture.

mod pg;
mod stub;
mod amqp;

pub use self::pg::{PgCacheAdapter, PgQuery, PgStoreAdapter};
pub use self::stub::StubEmitterAdapter;
pub use self::amqp::{AMQPEmitterAdapter};
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Serialize};
use Aggregator;
use Event;
use Events;
use EventData;
use StoreQuery;
use futures::future::Future;
use std::io;

/// Storage backend
pub trait StoreAdapter<E: Events, Q: StoreQuery> {
    /// Read a list of events matching a query
    fn aggregate<T, A>(
        &self,
        query_args: A,
        since: Option<(T, DateTime<Utc>)>,
    ) -> Result<T, String>
    where
        T: Aggregator<E, A, Q> + Default,
        A: Clone;

    /// Save an event to the store
    fn save(&self, event: &Event<E>) -> Result<(), String>;
}

/// Result of a cache search
pub type CacheResult<T> = (T, DateTime<Utc>);

/// Caching backend
pub trait CacheAdapter<K> {
    /// Insert an item into the cache
    fn insert<V>(&self, key: &K, value: V)
    where
        V: Serialize;

    /// Retrieve an item from the cache
    fn get<T>(&self, key: &K) -> Option<CacheResult<T>>
    where
        T: DeserializeOwned;
}

/// Closure called when an incoming event must be handled

/// Event emitter interface
pub trait EmitterAdapter {
    /// Get all subscribed handlers
    fn get_subscriptions(&self) -> Vec<String>;

    /// Emit an event
    fn emit<E: Events + Sync>(&self, event: &Event<E>) -> Box<Future<Item=(), Error=io::Error> + Send + Sync>;

    /// Subscribe to an event
    fn subscribe<ED, H>(&mut self, handler: H) -> Box<Future<Item=(), Error=io::Error> + Send> where ED: EventData + 'static, H: Fn(&Event<ED>) -> () + Send + Sync + 'static;

    /// Stop listening for an event
    fn unsubscribe<ED: EventData>(&mut self);
}
