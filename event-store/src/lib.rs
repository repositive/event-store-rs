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
pub mod testhelpers;
mod utils;

use adapters::{CacheAdapter, CacheResult, EmitterAdapter, StoreAdapter};
pub use aggregator::Aggregator;
use chrono::prelude::*;
pub use event::Event;
pub use event_context::EventContext;
pub use event_store_derive_internals::{EventData, Events};
use futures::future::Future;
use serde::{Deserialize, Serialize};
use std::io::Error;
use store::Store;
pub use store_query::StoreQuery;
use tokio::runtime::current_thread;
use utils::BoxedFuture;
use uuid::Uuid;

/// Main event store
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

impl Events for EventReplayRequested {
    fn event_type(&self) -> &'static str {
        "EventReplayRequested"
    }

    fn event_namespace(&self) -> &'static str {
        "event_store"
    }

    fn event_namespace_and_type(&self) -> &'static str {
        "event_store.EventReplayRequested"
    }
}

#[derive(Serialize, Deserialize)]
struct DummyEvent {}

impl<'a, E, Q, S, C, EM> Store<'a, E, Q, S, C, EM> for EventStore<S, C, EM>
where
    E: Events + Sync,
    Q: StoreQuery + Send + Sync,
    S: StoreAdapter<E, Q> + Send + Sync,
    C: CacheAdapter<Q> + Send + Sync,
    EM: EmitterAdapter + Send + Sync,
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
    fn aggregate<T, A>(&self, query_args: A) -> Result<T, String>
    where
        T: Aggregator<E, A, Q> + Serialize + for<'de> Deserialize<'de> + PartialEq,
        A: Clone,
    {
        let q = T::query(query_args.clone());
        let initial_state: Option<CacheResult<T>> = self.cache.get(&q);

        self.store
            .aggregate(query_args, initial_state.clone())
            .map(|agg| {
                if let Some((last_cache, _)) = initial_state {
                    // Only update cache if aggregation result has changed
                    if agg != last_cache {
                        self.cache.insert(&q, agg.clone());
                    }
                } else {
                    // If there is no existing cache item, insert one
                    self.cache.insert(&q, agg.clone());
                }

                agg
            })
    }

    /// Save an event to the store with optional context
    fn save(&self, event: Event<E>) -> Result<(), String> {
        self.store.save(&event)?;
        current_thread::block_on_all(self.emitter.emit(&event))
            .map_err(|_| "It was not possible to emit the event".into())
    }

    fn subscribe<ED, H>(&self, handler: H) -> BoxedFuture<(), Error>
    where
        ED: EventData + 'static,
        H: Fn(&Event<ED>) -> () + Send + Sync + 'static,
    {
        Box::new(self.emitter.subscribe(handler).and_then(move |_| {
            let event = Event::new(
                EventReplayRequested {
                    requested_event_type: ED::event_type().into(),
                    requested_event_namespace: ED::event_namespace().into(),
                    since: Utc::now(),
                },
                Uuid::new_v4(),
                EventContext {
                    action: None,
                    subject: None,
                    time: Utc::now(),
                },
            );
            self.emitter.emit(&event)
        }))
    }
}
