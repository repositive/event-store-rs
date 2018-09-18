//! Adapters for use with event store integrations
//!
//! A store will require a storage backend, cache backend and an event emitter instance for
//! integration with other event-driven domains. Use the adapters exposed by this module to suit
//! your application and architecture.

mod amqp;
mod pg;
mod stub;

pub use self::amqp::AMQPEmitterAdapter;
pub use self::pg::{PgCacheAdapter, PgQuery, PgStoreAdapter};
pub use self::stub::StubEmitterAdapter;
use chrono::{DateTime, Utc};
use event_store_derive_internals::EventData;
use serde::{de::DeserializeOwned, Serialize};
use std::io;
use utils::BoxedFuture;
use Event;
use Events;
use StoreQuery;

/// Storage backend
pub trait StoreAdapter<Q: StoreQuery>: Send + Sync + Clone + 'static {
    /// Read a list of events matching a query

    fn read<'b, E>(
        &self,
        query: Q,
        since: Option<DateTime<Utc>>,
    ) -> BoxedFuture<'b, Vec<E>, String>
    where
        E: Events + Send + 'b,
        Q: 'b;
    /// Save an event to the store
    fn save<'b, ED>(&self, event: &'b Event<ED>) -> BoxedFuture<'b, (), String>
    where
        ED: EventData + Send + Sync + 'b;

    /// Returns the last event of the type ED
    fn last_event<'b, ED: EventData + Send + 'b>(
        &self,
    ) -> BoxedFuture<'b, Option<Event<ED>>, String>;
}

/// Result of a cache search
pub type CacheResult<T> = (T, DateTime<Utc>);

/// Caching backend
pub trait CacheAdapter<K>: Send + Sync + Clone + 'static {
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
pub trait EmitterAdapter: Send + Sync + Clone + 'static {
    /// Emit an event
    fn emit<'a, E: EventData + Sync>(&self, event: &Event<E>) -> BoxedFuture<'a, (), io::Error>;

    /// Subscribe to an event
    fn subscribe<'a, ED, H>(&self, handler: H) -> BoxedFuture<'a, (), io::Error>
    where
        ED: EventData + 'a,
        H: Fn(&Event<ED>) -> () + Send + Sync + 'static;
}
