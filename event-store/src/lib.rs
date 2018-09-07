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
pub mod testhelpers;
mod utils;

use adapters::{CacheAdapter, CacheResult, EmitterAdapter, StoreAdapter};
use chrono::prelude::*;
pub use event_store_derive_internals::{EventData, Events};
use futures::future::{ok as FutOk, Future};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fmt::Debug;
use std::io::Error;
use tokio::runtime::current_thread;
use utils::BoxedFuture;
use uuid::Uuid;

/// Event context
///
/// Contains metadata for event and, most importantly, the creation time
#[derive(Serialize, Deserialize)]
pub struct EventContext {
    /// TODO: What is this?
    action: Option<String>,

    /// Optional event "subject" or metadata
    subject: Option<JsonValue>,

    /// Event creation time
    time: DateTime<Utc>,
}

/// Event with data, context and ID
///
/// This is what gets stored in the store and emitted from the emitter
#[derive(Serialize, Deserialize)]
pub struct Event<D> {
    data: D,
    context: EventContext,
    id: Uuid,
}

impl<D> Event<D> {
    /// Get the ID of this event
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the data of this event
    pub fn data(&self) -> &D {
        &self.data
    }

    /// Get the context of this event
    pub fn context(&self) -> &EventContext {
        &self.context
    }

    /// Create a new event
    pub fn new(data: D, id: Uuid, context: EventContext) -> Self {
        Self { data, context, id }
    }

    /// Create a new event from some data. `context.time` is set to now, `id` to a new V4 ID
    ///
    /// The rest of the context is left empty
    pub fn from_data(data: D) -> Self {
        Self {
            data,
            id: Uuid::new_v4(),
            context: EventContext {
                action: None,
                subject: None,
                time: Utc::now(),
            },
        }
    }

    /// Create a copied event with the given ID
    ///
    /// ```
    /// # extern crate uuid;
    /// # extern crate event_store;
    /// # use uuid::Uuid;
    /// # use event_store::testhelpers::*;
    /// # use event_store::Event;
    /// # let example_data = TestEvents::Inc(TestIncrementEvent {
    /// #     by: 1,
    /// #     ident: "it_aggregates_events".into(),
    /// # });
    /// #
    /// let event_id = Uuid::new_v4();
    /// let evt = Event::from_data(example_data).with_id(event_id);
    ///
    /// assert_eq!(evt.id(), event_id);
    /// ```
    pub fn with_id(self, id: Uuid) -> Self {
        Self { id, ..self }
    }
}

/// A query to be passed to the store
///
/// This trait must be implemented for whichever type you want to pass to a particular store. See
/// impls below for examples.
pub trait StoreQuery {}

/// Aggregator trait
///
/// This takes three type items:
///
/// * `E: EventData` – The enum of domain events that rows from the backing store will be parsed to
/// * `A` – The type of the query args to use when querying the backing store. Can be as simple as
/// a `String`, but using a `struct` is recommended for readability
/// * `Q: StoreQuery` – The query object to pass to the backing store. It should be built from `A`
///
/// `Aggregator` has trait bounds of `Copy + Clone + Debug + Default`. All can be `derive()`d easily
/// except `Default`, but that's easy enough to implement. `Default` should be the initial state
/// of the entity before the events are reduced onto it. Example:
///
/// ```rust
/// #[derive(Clone, Debug)]
/// struct ExampleUser {
///     name: String,
///     email: String,
///     bio: Option<String>
/// }
///
/// impl Default for ExampleUser {
///     fn default() -> Self {
///         Self {
///             name: "".into(),
///             email: "".into(),
///             bio: None,
///         }
///     }
/// }
/// ```
pub trait Aggregator<E: Events, A: Clone, Q: StoreQuery>: Clone + Debug + Default {
    /// Apply an event `E` to `acc`, returning a copy of `Self` with updated fields. Can also just
    /// return `acc` if nothing has changed.
    fn apply_event(acc: Self, event: &Event<E>) -> Self;

    /// Produce a query object from some query arguments
    fn query(field: A) -> Q;
}

/// Store trait
///
/// Backing stores must implement this trait to maintain portability. Additional bounds can be
/// added to `E: Events`. For example, the Postgres store implements `Store` with
/// `Events: Events + DeserializeOwned` so that the event data can be deserialized by Serde:
///
/// ```ignore
/// impl<'a, E> Store<E, PgQuery<'a>> for PgStore<E>
/// where
///     E: Events + DeserializeOwned,
/// {
///     // ...
/// }
/// ```
pub trait Store<
    'a,
    Q: StoreQuery + Send + Sync,
    S: StoreAdapter<Q> + Send + Sync,
    C,
    EM: EmitterAdapter + Send + Sync,
>: Send + Sync + 'static
{
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self;

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<E, T, A>(&self, query: A) -> Result<T, String>
    where
        E: Events,
        T: Aggregator<E, A, Q> + Serialize + for<'de> Deserialize<'de> + PartialEq,
        A: Clone;

    /// Save an event to the store with optional context
    fn save<ED: EventData + Send + Sync>(&self, event: Event<ED>) -> Result<(), String>;

    /// Subscribes the store to some events.
    fn subscribe<ED, H>(&self, handler: H) -> BoxedFuture<(), Error>
    where
        ED: EventData + Send + 'static,
        H: Fn(&Event<ED>) -> () + Send + Sync + 'static;
}

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

    fn subscribe<ED, H>(&self, handler: H) -> BoxedFuture<(), Error>
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
                }),
        )
    }
}
