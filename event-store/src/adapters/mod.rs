//! Adapters for use with event store integrations
//!
//! A store will require a storage backend, cache backend and an event emitter instance for
//! integration with other event-driven domains. Use the adapters exposed by this module to suit
//! your application and architecture.

// TODO: No pub
pub mod amqp;
mod pg;
mod redis;
mod stub;

pub use self::amqp::AMQPEmitterAdapter;
pub use self::pg::{PgCacheAdapter, PgQuery, PgStoreAdapter};
pub use self::redis::RedisCacheAdapter;
pub use self::stub::StubEmitterAdapter;
use chrono::{DateTime, Utc};
use event_store_derive_internals::EventData;
use serde::{de::DeserializeOwned, Serialize};
use std::io;
use std::thread::JoinHandle;
use Event;
use Events;
use StoreQuery;

/// Storage backend
pub trait StoreAdapter<Q: StoreQuery>: Send + Sync + Clone + 'static {
    /// Read a list of events matching a query

    fn read<E>(&self, query: Q, since: Option<DateTime<Utc>>) -> Result<Vec<E>, String>
    where
        E: Events + Send;
    /// Save an event to the store
    fn save<ED>(&self, event: &Event<ED>) -> Result<(), String>
    where
        ED: EventData + Send;

    /// Returns the last event of the type ED
    fn last_event<ED>(&self) -> Result<Option<Event<ED>>, String>
    where
        ED: EventData + Send;
}

/// Result of a cache search
pub type CacheResult<T> = (T, DateTime<Utc>);

/// Caching backend
pub trait CacheAdapter: Clone + Send + Sync + 'static {
    /// Insert an item into the cache
    fn set<V>(&self, key: String, value: V) -> Result<(), String>
    where
        V: Serialize + Send;

    /// Retrieve an item from the cache
    fn get<T>(&self, key: String) -> Result<Option<CacheResult<T>>, String>
    where
        T: DeserializeOwned + Send;
}

/// Closure called when an incoming event must be handled

/// Event emitter interface
pub trait EmitterAdapter: Clone + Send + Sync + 'static {
    /// Emit an event
    fn emit<E: EventData + Send>(&self, event: &Event<E>) -> Result<(), io::Error>;

    /// Subscribe to an event
    fn subscribe<ED, H>(&self, handler: H) -> JoinHandle<()>
    where
        ED: EventData + 'static,
        H: Fn(Event<ED>) -> () + Send + Sync + 'static;
}
