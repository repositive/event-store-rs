use adapters::{EmitterAdapter, StoreAdapter};
use aggregator::Aggregator;
use serde::Serialize;
use store_query::StoreQuery;
use utils::BoxedFuture;
use Event;
use EventData;
use Events;

/// Store trait
pub trait Store<
    'a,
    SQ: StoreQuery<'a> + 'a,
    S: StoreAdapter<'a, SQ> + Send + Sync,
    C,
    EM: EmitterAdapter + Send + Sync,
>
{
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self;

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<'b, E, A, T>(&self, query: A) -> BoxedFuture<'a, Option<T>, String>
    where
        E: Events,
        A: Serialize,
        T: Aggregator<'a, E, A, SQ>,
        A: Clone;

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
