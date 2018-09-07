use adapters::{EmitterAdapter, StoreAdapter};
use aggregator::Aggregator;
use event::Event;
use event_store_derive_internals::{EventData, Events};
use serde::{Deserialize, Serialize};
use store_query::StoreQuery;
use utils::BoxedFuture;

/// Store trait
pub trait Store<'a, Q: StoreQuery, S: StoreAdapter<Q>, C, EM: EmitterAdapter> {
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self;

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<E, T, A>(&self, query: A) -> Result<T, String>
    where
        E: Events,
        T: Aggregator<E, A, Q> + Serialize + for<'de> Deserialize<'de> + PartialEq,
        A: Clone;

    /// Save an event to the store with optional context
    fn save<ED: EventData + Send + Sync>(&self, event: Event<ED>) -> Result<(), String>;

    /// Subscribe to an event
    fn subscribe<ED, H>(&self, handler: H) -> BoxedFuture<(), String>
    where
        ED: EventData + Send + 'static,
        H: Fn(&Event<ED>) -> () + Send + Sync + 'static;
}
