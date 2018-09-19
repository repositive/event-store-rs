use adapters::{CacheAdapter, EmitterAdapter, StoreAdapter};
use aggregator::Aggregator;
use event::Event;
use event_store_derive_internals::{EventData, Events};
use serde::{Deserialize, Serialize};
use store_query::StoreQuery;
use utils::BoxedFuture;

/// Store trait
pub trait Store<
    'a,
    Q: StoreQuery + Send + Sync + 'a,
    S: StoreAdapter<Q> + Send + Sync,
    C: CacheAdapter + 'a,
    EM: EmitterAdapter + Send + Sync,
>: Send + Sync + 'a
{
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self;

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<'b, E, T, A>(&'b self, query: A) -> BoxedFuture<'b, T, String>
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
    fn save<'b, ED: EventData + Send + Sync + 'b>(
        &'b self,
        event: &'b Event<ED>,
    ) -> BoxedFuture<'b, (), String>;

    /// Subscribe to an event
    fn subscribe<'b, ED, H>(&'b self, handler: H) -> BoxedFuture<'b, (), String>
    where
        ED: EventData + Send + Sync + 'b,
        H: Fn(&Event<ED>) -> () + Send + Sync + 'static;
}
