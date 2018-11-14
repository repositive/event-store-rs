//! Event store for working with event-store-driven applications
#![deny(missing_docs)]

extern crate fallible_iterator;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
#[macro_use]
extern crate event_store_derive;
extern crate event_store_derive_internals;
extern crate postgres;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate sha2;
extern crate uuid;
#[macro_use]
extern crate log;
extern crate futures;
extern crate lapin_futures as lapin;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate tokio;

pub mod adapters;
mod aggregator;
mod event;
mod event_context;
pub mod prelude;
mod store;
mod store_query;
#[macro_use]
pub mod testhelpers;
mod utils;

use adapters::{CacheAdapter, CacheResult, EmitterAdapter, StoreAdapter};
use aggregator::Aggregator;
use chrono::prelude::*;
pub use event::Event;
pub use event_context::EventContext;
use event_store_derive_internals::{EventData, Events};
use std::thread::JoinHandle;
use store::Store;
use store_query::StoreQuery;

/// Main event store
#[derive(Clone)]
pub struct EventStore<S, C, EM> {
    store: S,
    cache: C,
    emitter: EM,
}

#[derive(Serialize, Deserialize)]
struct EventReplayRequested {
    requested_event_type: String,
    requested_event_namespace: String,
    since: DateTime<Utc>,
}

impl EventData for EventReplayRequested {
    fn event_type() -> &'static str {
        "EventReplayRequested"
    }

    fn event_namespace() -> &'static str {
        "event_store"
    }

    fn event_namespace_and_type() -> &'static str {
        "event_store.EventReplayRequested"
    }
}

#[derive(Serialize, Deserialize)]
struct DummyEvent {}

impl<Q, S, C, EM> Store<Q, S, C, EM> for EventStore<S, C, EM>
where
    Q: StoreQuery,
    S: StoreAdapter<Q>,
    C: CacheAdapter,
    EM: EmitterAdapter,
{
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self {
        Self {
            store,
            cache,
            emitter,
        }
    }

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<E, T, A>(&self, query_args: A) -> Result<T, String>
    where
        E: Events + Send,
        T: Aggregator<E, A, Q>,
        A: Clone,
    {
        let store_query = T::query(query_args.clone());
        let cache_id = store_query.unique_id();

        let cache_result: Option<CacheResult<T>> = self.cache.get(cache_id.clone())?;

        let initial_state = cache_result
            .clone()
            .map_or(T::default(), |(state, _)| state);
        let since = cache_result.clone().map(|(_, since)| since);
        let event_list = self.store.read(store_query, since)?;

        let agg = event_list
            .iter()
            .fold(initial_state, |acc, event| T::apply_event(acc, &event));

        match cache_result {
            Some((ref cache_state, _)) if &agg != cache_state => {
                self.cache.set(cache_id, agg.clone())
            }
            None => self.cache.set(cache_id, agg.clone()),
            _ => Ok(()),
        }?;

        Ok(agg)
    }

    /// Save an event to the store with optional context
    fn save<ED>(&self, event: &Event<ED>) -> Result<(), String>
    where
        ED: EventData + Send,
    {
        self.store.save(event)?;

        self.emitter
            .emit(&event)
            .map_err(|_| "It was not possible to emit the event".into())
    }

    fn subscribe<ED, H>(&self, handler: H) -> Result<JoinHandle<()>, ()>
    where
        ED: EventData + Send + Sync + 'static,
        H: Fn(Event<ED>, &Self) -> () + Send + Sync + 'static,
    {
        let _self = self.clone();

        let handle = self.emitter.subscribe(move |event: Event<ED>| {
            let event_id = event.id;

            trace!("Subscription received event ID {}", event_id);

            _self
                .store
                .save(&event)
                .map(|_| {
                    handler(event, &_self);
                })
                .expect(&format!("Failed to handle event with ID {}", event_id));
        });

        Ok(handle)
    }
}
