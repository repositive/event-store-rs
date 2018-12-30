use crate::adapters::{
    AmqpEmitterAdapter, PgCacheAdapter, PgQuery, PgStoreAdapter, SubscribeOptions,
};
use crate::aggregator::Aggregator;
use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::event_replay::EventReplayRequested;
use crate::store::Store;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use log::info;
use std::fmt::Debug;
use std::io;

#[derive(Clone)]
pub struct SubscribableStore {
    emitter: AmqpEmitterAdapter,
    inner_store: Store,
}

impl SubscribableStore {
    pub async fn new(
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

        await!(store.subscribe::<EventReplayRequested>(SubscribeOptions {
            replay_previous_events: false,
            save_on_receive: false
        }))?;

        Ok(store)
    }

    pub async fn aggregate<'a, T, QA, E>(&'a self, query_args: &'a QA) -> Result<T, io::Error>
    where
        E: Events,
        T: Aggregator<E, QA, PgQuery>,
        QA: Clone + Debug + 'a,
    {
        let res: T = await!(self.inner_store.aggregate::<'a, T, QA, E>(&query_args))?;

        Ok(res)
    }

    pub async fn save<'a, ED>(&'a self, event: &'a Event<ED>) -> Result<(), io::Error>
    where
        ED: EventData + Debug + Send + Sync,
    {
        await!(self.inner_store.save(event))
    }

    pub async fn save_no_emit<'a, ED>(&'a self, event: &'a Event<ED>) -> Result<(), io::Error>
    where
        ED: EventData + Debug + Send + Sync,
    {
        await!(self.inner_store.save_no_emit(event))
    }

    pub async fn subscribe<'a, ED>(&'a self, options: SubscribeOptions) -> Result<(), io::Error>
    where
        ED: EventHandler + Debug + Send + Sync,
    {
        info!(
            "Starting subscription to {}",
            ED::event_namespace_and_type()
        );

        let inner_store = self.inner_store.clone();

        await!(self.emitter.subscribe::<ED>(inner_store, options))?;

        Ok(())
    }
}
