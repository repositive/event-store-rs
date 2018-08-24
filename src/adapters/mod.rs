//! Adapters for use with event store integrations
//!
//! A store will require a storage backend, cache backend and an event emitter instance for
//! integration with other event-driven domains. Use the adapters exposed by this module to suit
//! your application and architecture.

pub mod pg;

use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use Events;

/// Storage backend
pub trait StoreAdapter<E: Events, Q> {
    /// Read a list of events matching a query
    fn read(&self, query: Q, since: Option<DateTime<Utc>>) -> Result<Vec<Box<E>>, String>;

    /// Save an event to the store
    fn save(&self, event: E) -> Result<(), String>;
}

/// Caching backend
pub trait CacheAdapter<K> {
    /// Insert an item into the cache
    fn insert<V>(&self, key: &K, value: V)
    where
        V: Serialize;

    /// Retrieve an item from the cache
    fn get<V>(&self, key: &K) -> Option<(V, DateTime<Utc>)>
    where
        V: DeserializeOwned;
}

/// Event emitter interface
// TODO: Trait bounds on handler type, Fn() -> () or whatever
pub trait EmitterAdapter<E: Events, H> {
    /// Get a list of subscriptions
    fn get_subscriptions(&self) -> HashMap<String, H>;

    /// Emit an event
    fn emit(&self, event: E);

    /// Subscribe to an event
    fn subscribe(&mut self, event_name: String, handler: H);

    /// Stop listening for an event
    fn unsubscribe(&mut self, event_name: String);
}
