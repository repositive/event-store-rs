use adapters::{CacheAdapter, EmitterAdapter, StoreAdapter};
use aggregator::Aggregator;
use event::Event;
use event_store_derive_internals::{EventData, Events};
use serde::{Deserialize, Serialize};
use std::thread::JoinHandle;
use store_query::StoreQuery;

/// Store trait
pub trait Store<Q, S, C, EM>: Clone
where
    Q: StoreQuery,
    S: StoreAdapter<Q>,
    C: CacheAdapter + Clone + Send + Sync + 'static,
    EM: EmitterAdapter + Send + Sync + 'static,
{
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self;

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<E, T, A>(&self, query: A) -> Result<T, String>
    where
        E: Events + Send,
        T: Aggregator<E, A, Q> + Serialize + for<'de> Deserialize<'de> + PartialEq + Send,
        A: Clone;

    /// Save an event to the store with optional context
    fn save<ED>(&self, event: &Event<ED>) -> Result<(), String>
    where
        ED: EventData + Send;

    /// Subscribe to an event
    fn subscribe<ED, H>(&self, handler: H) -> Result<JoinHandle<()>, ()>
    where
        ED: EventData + Send + Sync + 'static,
        H: Fn(Event<ED>, &Self) -> () + Send + Sync + 'static;
}
