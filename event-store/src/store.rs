use adapters::amqp::AMQPSender;
use adapters::{CacheAdapter, EmitterAdapter, StoreAdapter};
use aggregator::Aggregator;
use event::Event;
use event_store_derive_internals::{EventData, Events};
use serde::{Deserialize, Serialize};
use std::thread::JoinHandle;
use store_query::StoreQuery;
use tokio::runtime::Runtime;

/// Store trait
pub trait Store<
    'a,
    Q: StoreQuery + Send + Sync + 'a,
    S: StoreAdapter<Q> + Send + Sync,
    C: CacheAdapter + 'static,
    // EM: EmitterAdapter + Send + Sync,
>: Send + Sync + Clone + 'a
{
    /// Create a new event store
    fn new(store: S, cache: C, sender: AMQPSender) -> Self;

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
    fn save<'b, ED: EventData + Send + Sync + 'b>(
        &'b self,
        event: &'b Event<ED>,
    ) -> Result<(), String>;

    // /// Subscribe to an event
    // fn subscribe<ED, H>(&self, handler: H) -> Result<Runtime, ()>
    // where
    //     ED: EventData + Send + Sync + 'static,
    //     H: Fn(&Event<ED>, &InnerStore<S, C>) -> () + Send + Sync + 'static;
}
