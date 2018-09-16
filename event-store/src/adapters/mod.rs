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
use utils::{BoxedFuture, BoxedStream};
use Event;
use Events;

/// Storage backend
pub trait StoreAdapter {
    /// Reads a list of events from the db
    fn read<'a, E: Events + Sync + Send + 'a, A: Clone>(
        &self,
        query_args: A,
        since: Utc,
    ) -> BoxedStream<'a, E, String>;
    /// Save an event to the store
    fn save<'a, ED: EventData + 'a>(&self, event: Event<ED>) -> BoxedFuture<'a, (), String>;

    /// Returns the last event of the type ED
    fn last_event<'a, ED: EventData + Send + 'a>(
        &self,
    ) -> BoxedFuture<'a, Option<Event<ED>>, String>;
}

/// Result of a cache search
pub type CacheResult<T> = (T, DateTime<Utc>);

/// Caching backend
pub trait CacheAdapter {
    /// Insert an item into the cache
    fn insert<V>(&self, key: String, value: V) -> BoxedFuture<(), String>
    where
        V: Serialize;

    /// Retrieve an item from the cache
    fn get<'a, T: Sync + Send + DeserializeOwned + 'a>(
        &self,
        key: String,
    ) -> BoxedFuture<'a, Option<CacheResult<T>>, String>
    where
        T: DeserializeOwned;
}

/// Closure called when an incoming event must be handled

/// Event emitter interface
pub trait EmitterAdapter {
    /// Emit an event
    fn emit<'a, E: EventData>(&self, event: &Event<E>) -> BoxedFuture<'a, (), io::Error>;

    /// Subscribe to an event
    fn subscribe<'a, ED, H>(&self, handler: H) -> BoxedFuture<'a, (), io::Error>
    where
        ED: EventData + 'a,
        H: Fn(&Event<ED>) -> () + Send + Sync + 'static;
}
