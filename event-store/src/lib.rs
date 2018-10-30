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
use futures::lazy;
use serde::{Deserialize, Serialize};
use std::thread;
use std::thread::JoinHandle;
use store::Store;
use store_query::StoreQuery;
use tokio::runtime::current_thread::block_on_all;
use tokio::runtime::Runtime;

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

impl<'a, Q, S, C, EM> Store<'a, Q, S, C, EM> for EventStore<S, C, EM>
where
    Q: StoreQuery + Send + Sync + 'a,
    S: StoreAdapter<Q> + Send + Sync + Clone + 'a,
    C: CacheAdapter + Send + Sync + Clone + 'static,
    EM: EmitterAdapter + Send + Sync + Clone + 'a,
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
    fn aggregate<'b, E, T, A>(&'b self, query_args: A) -> Result<T, String>
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
        A: Clone + 'b,
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
    fn save<'b, ED: EventData + Send + Sync + 'b>(
        &'b self,
        event: &'b Event<ED>,
    ) -> Result<(), String> {
        self.store.save(event)?;

        self.emitter
            .emit(&event)
            .map_err(|_| "It was not possible to emit the event".into())
    }

    fn subscribe<ED, H>(&self, handler: H) -> Result<Runtime, ()>
    where
        ED: EventData + Send + Sync + 'static,
        H: Fn(&Event<ED>, &Self) -> () + Send + Sync + 'static,
    {
        let _self = self.clone();
        let handler_store = self.store.clone();

        let sub = self.emitter.subscribe(move |event: &Event<ED>| {
            trace!("Subscription received event ID {}", event.id);

            let _ = handler_store.save(event).map(|_| {
                handler(event, &_self);
            });
        });

        tokio::run(futures::lazy(|| {
            tokio::spawn(sub);

            Ok(())
        }));

        Ok(Runtime::new().unwrap())
    }
}
