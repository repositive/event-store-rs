mod pg;

use chrono::{DateTime, Utc};
use event::Event;
use event_store_derive_internals::{EventData, Events};
use serde_json::Value as JsonValue;
use std::io;
use utils::BoxedFuture;

pub use self::pg::{PgQuery, PgStoreAdapter};

/// Storage backend
pub trait StoreAdapter<Q: StoreQuery>: Send + Clone + 'static {
    /// Read a list of events matching a query

    fn read<E>(&self, query: Q, since: Option<DateTime<Utc>>) -> BoxedFuture<Vec<E>, io::Error>
    where
        E: Events + Send + 'static;
    /// Save an event to the store
    fn save<ED>(&self, event: &Event<ED>) -> Result<(), String>
    where
        ED: EventData;

    /// Returns the last event of the type ED
    fn last_event<ED>(&self) -> Result<Option<Event<ED>>, String>
    where
        ED: EventData;

    /// Reads events created at or after a specific time given an event type and namespace
    ///
    /// Unlike other parts of the API, this method requires passing of a namespace and event name
    /// string, which may lead to incorrect behaviour due to typos, etc.
    fn read_events_since(
        &self,
        event_namespace: String,
        event_type: String,
        since: DateTime<Utc>,
    ) -> Result<Vec<JsonValue>, String>;
}

/// A query to be passed to the store
///
/// This trait must be implemented for whichever type you want to pass to a particular store. See
/// impls below for examples.
pub trait StoreQuery: Send + 'static {
    /// You must return a unique identifier based on the query you are performing. This identifier
    /// will then be used to identify the cache and optimize the aggregations using memoization
    fn unique_id(&self) -> String;
}
