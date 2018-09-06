use adapters::{EmitterAdapter, StoreAdapter};
use aggregator::Aggregator;
use event::Event;
use event_store_derive_internals::Events;
use serde::{Deserialize, Serialize};
use store_query::StoreQuery;

/// Store trait
pub trait Store<'a, E: Events, Q: StoreQuery, S: StoreAdapter<E, Q>, C, EM: EmitterAdapter> {
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self;

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<T, A>(&self, query: A) -> Result<T, String>
    where
        T: Aggregator<E, A, Q> + Serialize + for<'de> Deserialize<'de> + PartialEq,
        A: Clone;

    /// Save an event to the store with optional context
    fn save(&self, event: Event<E>) -> Result<(), String>;
}