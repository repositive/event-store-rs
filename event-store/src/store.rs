use adapters::{CacheAdapter, EmitterAdapter, StoreAdapter, StoreQuery};
use aggregator::Aggregator;
use event::Event;
use event_store_derive_internals::{EventData, Events};
use serde::{Deserialize, Serialize};
use std::io;
use utils::BoxedFuture;

/// Store trait
pub trait Store<Q, S, C, EM>: Clone
where
    Q: StoreQuery,
    S: StoreAdapter<Q>,
    C: CacheAdapter,
    EM: EmitterAdapter,
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
    fn save<ED>(&self, event: &Event<ED>) -> BoxedFuture<(), io::Error>
    where
        ED: EventData + Send;

    /// Subscribe to an event
    fn subscribe<ED, H>(&self, handler: H) -> BoxedFuture<(), io::Error>
    where
        ED: EventData + Send + Sync + 'static,
        H: Fn(Event<ED>, &Self) -> () + Send + Sync + 'static;
}
