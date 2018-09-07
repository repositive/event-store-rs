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
pub use event::Event;
pub use event_context::EventContext;
use event_store_derive_internals::{Events, EventData};
use serde::{Deserialize, Serialize};
use store::Store;
pub use store_query::StoreQuery;
use chrono::prelude::*;
use tokio::runtime::current_thread;
use futures::future::{Future, ok as FutOk};
use uuid::Uuid;
use utils::BoxedFuture;

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

impl<'a, Q, S, C, EM> Store<'a, Q, S, C, EM> for EventStore<S, C, EM>
where
    Q: StoreQuery + Send + Sync,
    S: StoreAdapter<Q> + Send + Sync + Clone + 'static,
    C: CacheAdapter<Q> + Send + Sync + Clone + 'static,
    EM: EmitterAdapter + Send + Sync + Clone + 'static,
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
        E: Events,
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
    fn save<ED: EventData + Send + Sync>(&self, event: Event<ED>) -> Result<(), String> {
        self.store.save(&event)?;
        current_thread::block_on_all(self.emitter.emit(&event))
            .map_err(|_| "It was not possible to emit the event".into())
    }

    fn subscribe<ED, H>(&self, handler: H) -> BoxedFuture<(), String>
    where
        ED: EventData + Send + 'static,
        H: Fn(&Event<ED>) -> () + Send + Sync + 'static,
    {
        let handler_store = self.store.clone();
        Box::new(
            self.emitter
                .subscribe(move |event: &Event<ED>| {
                    let _ = handler_store.save(event).map(|_| {
                        handler(event);
                    });
                    /**/
                }).and_then(move |_| {
                    self.store
                        .last_event::<ED>()
                        .map(|o_event| {
                            o_event
                                .map(|event| event.context.time)
                                .unwrap_or_else(|| Utc::now())
                        }).or_else(|_| FutOk(Utc::now()))
                }).and_then(move |since| {
                    let data = EventReplayRequested {
                        requested_event_type: ED::event_type().into(),
                        requested_event_namespace: ED::event_namespace().into(),
                        since,
                    };
                    let id = Uuid::new_v4();
                    let context = EventContext {
                        action: None,
                        subject: None,
                        time: Utc::now(),
                    };
                    let event = Event { data, id, context };
                    self.emitter.emit(&event)
                })
                .map_err(|_| "It was not possible to subscribe".into()),
        )
    }
}
