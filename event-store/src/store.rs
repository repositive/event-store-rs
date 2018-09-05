use adapters::{EmitterAdapter, StoreAdapter};
use aggregator::Aggregator;
use event::Event;
use event_store_derive_internals::{EventData, Events};
use serde::{Deserialize, Serialize};
use std::io::Error;
use store_query::StoreQuery;
use utils::BoxedFuture;

/// Store trait
pub trait Store<
    'a,
    E: Events,
    Q: StoreQuery + Send + Sync,
    S: StoreAdapter<E, Q> + Send + Sync,
    C,
    EM: EmitterAdapter + Send + Sync,
>
{
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self;

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<T, A>(&self, query: A) -> Result<T, String>
    where
        T: Aggregator<E, A, Q> + Serialize + for<'de> Deserialize<'de> + PartialEq,
        A: Clone;

    /// Save an event to the store with optional context
    fn save(&self, event: Event<E>) -> Result<(), String>;

    /// Subscribes the store to some events.
    fn subscribe<ED, H>(&self, handler: H) -> BoxedFuture<(), Error>
    where
        ED: EventData + Send + 'static,
        H: Fn(&Event<ED>) -> () + Send + Sync + 'static;
}
