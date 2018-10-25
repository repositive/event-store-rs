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
use futures::future::{ok as FutOk, Future};
use serde::{Deserialize, Serialize};
use store::Store;
use store_query::StoreQuery;
use tokio::runtime::current_thread::block_on_all;
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

#[derive(Serialize, Deserialize)]
struct DummyEvent {}

impl<'a, Q, S, C, EM> Store<'a, Q, S, C, EM> for EventStore<S, C, EM>
where
    Q: StoreQuery + Send + Sync + 'a,
    S: StoreAdapter<Q> + Send + Sync + Clone + 'a,
    C: CacheAdapter + Send + Sync + Clone + 'a,
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
    fn aggregate<'b, E, T, A>(&'b self, query_args: A) -> BoxedFuture<'b, T, String>
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

        Box::new(self.cache.get(cache_id.clone()).and_then(
            move |cache_result: Option<CacheResult<T>>| {
                let initial_state = cache_result
                    .clone()
                    .map_or(T::default(), |(state, _)| state);
                let since = cache_result.clone().map(|(_, since)| since);
                self.store
                    .read(store_query, since)
                    .and_then(move |event_list| {
                        let agg = event_list
                            .iter()
                            .fold(initial_state, |acc, event| T::apply_event(acc, &event));
                        let save_cache_task = match cache_result {
                            Some((ref cache_state, _)) if &agg != cache_state => {
                                self.cache.set(cache_id, agg.clone())
                            }
                            None => self.cache.set(cache_id, agg.clone()),
                            _ => Box::new(FutOk(())),
                        };
                        save_cache_task.and_then(|_| FutOk(agg))
                    })
            },
        ))
    }

    /// Save an event to the store with optional context
    fn save<'b, ED: EventData + Send + Sync + 'b>(
        &'b self,
        event: &'b Event<ED>,
    ) -> BoxedFuture<'b, (), String> {
        Box::from(self.store.save(event).and_then(move |_| {
            Box::new(
                self.emitter
                    .emit(&event)
                    .map_err(|_| "It was not possible to emit the event".into()),
            )
        }))
    }

    fn subscribe<ED, H>(&self, handler: H)
    where
        ED: EventData + Send + Sync,
        H: Fn(&Event<ED>) -> () + Send + Sync + 'static,
    {
        let handler_store = self.store.clone();

        block_on_all(
            self.emitter
                .subscribe(move |event: &Event<ED>| {
                    let _ = handler_store.save(event).map(|_| {
                        handler(event);
                    });
                    /**/
                })
                .and_then(move |_| {
                    self.store
                        .last_event::<ED>()
                        .map(|o_event| {
                            o_event
                                .map(|event| event.context.time)
                                .unwrap_or_else(|| Utc::now())
                        })
                        .or_else(|_| FutOk(Utc::now()))
                })
                .and_then(move |since| {
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
                }),
        )
        .expect("Could not subscribe");
    }
}
