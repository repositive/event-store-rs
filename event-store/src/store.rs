use adapters::{CacheAdapter, EmitterAdapter, StoreAdapter};
use aggregator::Aggregator;
use event::Event;
use event_store_derive_internals::{EventData, Events};
use serde::{Deserialize, Serialize};
use std::thread::JoinHandle;
use store_query::StoreQuery;

/// Store trait
pub trait Store<
    'a,
    Q: StoreQuery + Send + Sync + 'a,
    S: StoreAdapter<Q> + Send + Sync,
    C: CacheAdapter + 'static,
    EM: EmitterAdapter + Send + Sync,
>: Send + Sync + Clone + 'a
{
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self;

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<'b, E, T, A>(&'b self, query: A) -> Result<T, String>
    where
        E: Events + Send + Sync + 'b,
        Q: 'b,
        T: Aggregator<E, A, Q>
            + Send
            + Sync
            + Serialize
            + for<'de> Deserialize<'de>
            + PartialEq
            + 'b,
        A: Clone + 'b;

    /// Save an event to the store with optional context
    fn save<ED: EventData + Send + Sync>(&self, event: &Event<ED>) -> Result<(), String>;

    /// Subscribe to an event
    fn subscribe<ED, H>(&self, handler: H) -> Result<JoinHandle<()>, ()>
    where
        ED: EventData + Send + Sync + 'static,
        H: Fn(Event<ED>, &Self) -> () + Send + Sync + 'static;
}
