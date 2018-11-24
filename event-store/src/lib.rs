//! Event store for working with event-store-driven applications
#![deny(missing_docs)]

extern crate fallible_iterator;
extern crate redis;
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
#[macro_use]
pub mod testhelpers;
mod utils;

use adapters::{CacheAdapter, CacheResult, EmitterAdapter, StoreAdapter, StoreQuery};
use aggregator::Aggregator;
use chrono::prelude::*;
pub use event::Event;
pub use event_context::EventContext;
use event_store_derive_internals::{EventData, Events};
use futures::Future;
use std::io;
use store::Store;
use utils::BoxedFuture;

/// Main event store
#[derive(Clone)]
pub struct EventStore<S, C, EM> {
    store: S,
    cache: C,
    emitter: EM,
}

#[derive(EventData)]
#[event_store(namespace = "_eventstore")]
struct EventReplayRequested {
    requested_event_type: String,
    requested_event_namespace: String,
    since: DateTime<Utc>,
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
        let _store = store.clone();
        let _emitter = emitter.clone();

        trace!("Create new store");

        let replay_fut = _emitter
            .clone()
            .subscribe(move |replay: Event<EventReplayRequested>| {
                trace!("Received replay request {}", replay.id);

                let EventReplayRequested {
                    requested_event_namespace,
                    requested_event_type,
                    since,
                    ..
                } = replay.data;

                let events = _store
                    .read_events_since(
                        requested_event_namespace.clone(),
                        requested_event_type.clone(),
                        since,
                    )
                    .expect("Could not read events since");

                trace!("Found {} events to replay", events.len());

                for event in events {
                    trace!("Replay event {:?}", event);

                    tokio::spawn(
                        _emitter
                            .emit_with_string_ident(
                                &requested_event_namespace,
                                &requested_event_type,
                                &event,
                            )
                            .map_err(move |e| {
                                error!("Failed to replay event ID {}: {}", event["id"], e)
                            }),
                    );
                }
            })
            .map_err(|e| error!("Failed to subscribe to EventReplayRequested: {}", e));

        // TODO: Spawn this in a `Store.run()` call in the future. This needs to happen once the
        // `Store` is split into `InnerStore` and a subscribable `Store`.
        tokio::spawn(replay_fut);

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
        trace!("Begin aggregate");

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

        trace!("Aggregate complete");

        Ok(agg)
    }

    /// Save an event to the store with optional context
    fn save<ED>(&self, event: &Event<ED>) -> BoxedFuture<(), io::Error>
    where
        ED: EventData + Send,
    {
        self.store.save(event).expect("Save failed");

        Box::new(self.emitter.emit(&event))
    }

    fn subscribe<ED, H>(&self, handler: H) -> BoxedFuture<(), io::Error>
    where
        ED: EventData + Send + Sync + 'static,
        H: Fn(Event<ED>, &Self) -> () + Send + Sync + 'static,
    {
        let last_event: Option<Event<ED>> = self
            .store
            .last_event()
            .expect("Failed to look for last event");

        let _self = self.clone();
        let _self2 = self.clone();

        let fut = self
            .emitter
            .subscribe(move |event: Event<ED>| {
                let event_id = event.id;

                trace!("Subscription received event ID {}", event_id);

                _self
                    .store
                    .save(&event)
                    .map(|_| {
                        handler(event, &_self);
                    })
                    .unwrap_or_else(|_| panic!("Failed to handle event with ID {}", event_id));
            })
            .and_then(move |_| {
                _self2.emitter.emit(&Event::from_data(EventReplayRequested {
                    requested_event_type: ED::event_type().to_string(),
                    requested_event_namespace: ED::event_namespace().to_string(),
                    since: last_event.map(|e| e.context.time).unwrap_or_else(|| {
                        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
                    }),
                }))
            });

        Box::new(fut)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_meta<ED>(_event: ED) -> (&'static str, &'static str, &'static str)
    where
        ED: EventData,
    {
        (
            ED::event_namespace_and_type(),
            ED::event_namespace(),
            ED::event_type(),
        )
    }

    #[test]
    fn event_replay_requested_ident() {
        let res = get_meta(EventReplayRequested {
            requested_event_namespace: "some_ns".into(),
            requested_event_type: "SomeType".into(),
            since: Utc::now(),
        });

        assert_eq!(
            res,
            (
                "_eventstore.EventReplayRequested",
                "_eventstore",
                "EventReplayRequested"
            )
        );
    }
}
