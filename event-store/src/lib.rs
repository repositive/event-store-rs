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
use futures::future::lazy;
use futures::future::{ok as FutOk, Future};
use serde::{Deserialize, Serialize};
use std::thread::{self, JoinHandle};
use store::Store;
use store_query::StoreQuery;
use tokio::executor::current_thread;
use tokio::runtime::current_thread::{block_on_all, Runtime};
use utils::BoxedFuture;
use uuid::Uuid;

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
        };

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

    fn subscribe<ED, H>(&self, handler: H) -> Result<JoinHandle<()>, ()>
    where
        ED: EventData + Send + Sync + 'static,
        H: Fn(&Event<ED>) -> () + Send + Sync + 'static,
    {
        info!("Register");
        let handler_store = self.store.clone();
        let handler_store_2 = self.store.clone();
        let em = self.emitter.clone();

        // let res = lazy(move || {
        // let sub = ;

        let sub = em.subscribe(move |event: &Event<ED>| {
            info!("IDK");
            let _ = handler_store.save(event).map(|_| {
                handler(event);
            });
            /**/
        });

        let handle = thread::spawn(|| {
            let mut rt = Runtime::new().expect("Runtime could not be created");

            trace!("Spawn thread");
            rt.spawn(sub);
            trace!("RUN 1");
            // rt.spawn(listen2);
            // trace!("RUN 2");

            rt.run().expect("Runtime failed");

            trace!("Runtime RUNNING");
        });

        Ok(handle)

        // block_on_all(
        //     em.subscribe(move |event: &Event<ED>| {
        //         info!("IDK");
        //         let _ = handler_store.save(event).map(|_| {
        //             handler(event);
        //         });
        //         /**/
        //     })
        //     .map(|_| ())
        //     .map_err(|_| ()),
        // )
        // .expect("Block failed");

        // .and_then(move |_| {
        // info!("GOT HERE");
        // let last_event = handler_store_2
        //     .last_event::<ED>()
        //     .map(|o_event| {
        //         o_event
        //             .map(|event| event.context.time)
        //             .unwrap_or_else(|| Utc::now())
        //     })
        //     .or_else(|_| FutOk(Utc::now()))
        //     // })
        //     .and_then(move |since| {
        //         let data = EventReplayRequested {
        //             requested_event_type: ED::event_type().into(),
        //             requested_event_namespace: ED::event_namespace().into(),
        //             since,
        //         };
        //         let id = Uuid::new_v4();
        //         let context = EventContext {
        //             action: None,
        //             subject: None,
        //             time: Utc::now(),
        //         };
        //         let event = Event { data, id, context };
        //         em.emit(&event)
        //     });

        // Ok(res)
        // });
        //     Ok::<_, ()>(())
        // }))
        // .unwrap();
    }
}
