use adapters::{EmitterAdapter, StoreAdapter};
use aggregator::Aggregator;
use event::Event;
use event_store_derive_internals::{EventData, Events};
use serde::{Deserialize, Serialize};
use store_query::StoreQuery;
use utils::BoxedFuture;

/// Store trait
pub trait Store<
    'a,
    Q: StoreQuery + Send + Sync,
    S: StoreAdapter<Q> + Send + Sync,
    C,
    EM: EmitterAdapter + Send + Sync,
>: Send + Sync + 'a
{
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self;

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<'b, E, T, A>(&'b self, query: A) -> BoxedFuture<'b, T, String>
    where
        E: Events,
        T: Aggregator<E, A, Q> + Send + Serialize + for<'de> Deserialize<'de> + PartialEq + 'b,
        A: Clone + Send + 'b;

    /// Save an event to the store with optional context
    fn save<ED: EventData + Send + Sync + 'static>(
        &self,
        event: Event<ED>,
    ) -> BoxedFuture<(), String>;

    /// Subscribe to an event
    fn subscribe<ED, H>(&self, handler: H) -> BoxedFuture<(), String>
    where
        ED: EventData + Send + 'static,
        H: Fn(&Event<ED>) -> () + Send + Sync + 'static;
}
