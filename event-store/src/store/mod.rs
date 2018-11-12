use chrono::{DateTime, Utc};
use crate::event::Event;
use event_store_derive_internals::{EventData, Events};
use std::fmt::Debug;

pub mod pg;

/// A query to be passed to the store
///
/// This trait must be implemented for whichever type you want to pass to a particular store. See
/// impls below for examples.
pub trait StoreQuery {
    /// You must return a unique identifier based on the query you are performing. This identifier
    /// will then be used to identify the cache and optimize the aggregations using memoization
    fn unique_id(&self) -> String;
}

pub trait StoreAdapter<E, Q>
where
    E: Events + Debug,
    Q: StoreQuery,
{
    fn read(&self, query: Q, since: Option<DateTime<Utc>>) -> Result<Vec<E>, String>;

    fn save<ED>(&self, event: &Event<ED>) -> Result<(), String>
    where
        ED: EventData + Debug;

    fn last_event<ED>(&self) -> Result<Option<Event<ED>>, String>
    where
        ED: EventData + Debug;
}
