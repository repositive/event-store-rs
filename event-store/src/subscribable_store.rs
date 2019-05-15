use crate::adapters::{AmqpEmitterAdapter, PgCacheAdapter, PgQuery, PgStoreAdapter, SaveResult};
use crate::aggregator::Aggregator;
use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::store::Store;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use log::info;
use std::fmt::Debug;
use std::io;

/// The main event store struct
#[derive(Clone)]
pub struct SubscribableStore {
    emitter: AmqpEmitterAdapter,
    inner_store: Store,
}

impl SubscribableStore {
    /// Create a new event store with the given store, cache and emitter adapters
    pub fn new(
        store: PgStoreAdapter,
        cache: PgCacheAdapter,
        emitter: AmqpEmitterAdapter,
    ) -> Result<Self, io::Error> {
        // TODO: Pass these in as refs to Store
        let inner_store = Store::new(store, cache, emitter.clone());

        let store = Self {
            inner_store,
            emitter,
        };

        Ok(store)
    }

    /// Fetch an entity from the store by aggregating over matching events
    pub async fn aggregate<'a, T, QA, E>(&'a self, query_args: &'a QA) -> Result<T, io::Error>
    where
        E: Events,
        T: Aggregator<E, QA, PgQuery>,
        QA: Clone + Debug + 'a,
    {
        let res: T = await!(self.inner_store.aggregate::<'a, T, QA, E>(&query_args))?;

        Ok(res)
    }

    /// Save an event to the store, emitting it to other listeners
    pub async fn save<'a, ED>(&'a self, event: &'a Event<ED>) -> SaveResult
    where
        ED: EventData + Debug,
    {
        await!(self.inner_store.save(event))
    }

    /// Subscribe to incoming events matching the namespace and type in `ED`
    pub async fn subscribe<'a, ED>(&'a self) -> Result<(), io::Error>
    where
        ED: EventHandler + Debug + Send,
    {
        info!(
            "Starting subscription to {}",
            ED::event_namespace_and_type()
        );

        let inner_store = self.inner_store.clone();

        await!(self.emitter.subscribe::<ED>(inner_store))
    }

    // TODO: Can I do something clever with a trait impl here?
    /// Return a reference to the internal backing store. This is a dangerous method and should not
    /// be used in production code.
    pub fn internals_get_store(&self) -> &Store {
        &self.inner_store
    }
}
