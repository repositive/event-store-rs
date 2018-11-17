mod pg;

use chrono::{DateTime, Utc};
use event::Event;
use event_store_derive_internals::{EventData, Events};

pub use self::pg::{PgQuery, PgStoreAdapter};

/// Storage backend
pub trait StoreAdapter<Q: StoreQuery>: Send + Clone + 'static {
    /// Read a list of events matching a query

    fn read<E>(&self, query: Q, since: Option<DateTime<Utc>>) -> Result<Vec<E>, String>
    where
        E: Events;
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
    fn read_events_since<ED>(
        &self,
        event_namespace: String,
        event_type: String,
        since: DateTime<Utc>,
    ) -> Result<Vec<Event<ED>>, String>
    where
        ED: EventData;
}

/// A query to be passed to the store
///
/// This trait must be implemented for whichever type you want to pass to a particular store. See
/// impls below for examples.
pub trait StoreQuery {
    /// You must return a unique identifier based on the query you are performing. This identifier
    /// will then be used to identify the cache and optimize the aggregations using memoization
    fn unique_id(&self) -> String;
}
