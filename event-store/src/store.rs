use adapters::{EmitterAdapter, StoreAdapter};
use aggregator::Aggregator;
use store_query::StoreQuery;
use utils::BoxedFuture;
use Event;
use EventData;
use Events;

/// Store trait
pub trait Store<
    'a,
    Q: StoreQuery + Send + Sync,
    S: StoreAdapter + Send + Sync,
    C,
    EM: EmitterAdapter + Send + Sync,
>
{
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self;

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<'b, E, T, A>(&self, query: A) -> BoxedFuture<'b, Option<T>, String>
    where
        E: Events,
        T: Aggregator<'b, E, A, Q>,
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
