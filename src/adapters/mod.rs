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
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use Aggregator;
use Event;
use EventData;
use StoreQuery;

/// Storage backend
pub trait StoreAdapter<E: EventData, Q: StoreQuery> {
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
pub type EventHandler<E> = fn(&Event<E>) -> ();

/// Event emitter interface
pub trait EmitterAdapter<E: EventData> {
    /// Get all subscribed handlers
    fn get_subscriptions(&self) -> &HashMap<String, EventHandler<E>>;

    /// Emit an event
    fn emit(&self, event: &Event<E>);

    /// Subscribe to an event
    fn subscribe(&mut self, event_name: String, handler: EventHandler<E>);

    /// Stop listening for an event
    fn unsubscribe(&mut self, event_name: String);
}
