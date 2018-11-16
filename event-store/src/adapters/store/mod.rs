mod pg;

use chrono::{DateTime, Utc};
use event::Event;
use event_store_derive_internals::{EventData, Events};

pub use self::pg::{PgQuery, PgStoreAdapter};

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

/// A query to be passed to the store
///
/// This trait must be implemented for whichever type you want to pass to a particular store. See
/// impls below for examples.
pub trait StoreQuery {
    /// You must return a unique identifier based on the query you are performing. This identifier
    /// will then be used to identify the cache and optimize the aggregations using memoization
    fn unique_id(&self) -> String;
}
