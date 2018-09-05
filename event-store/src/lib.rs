//! Event store for working with event-store-driven applications
#![deny(missing_docs)]

extern crate fallible_iterator;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
#[macro_use]
extern crate event_store_derive;
extern crate event_store_derive_internals;
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
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fmt::Debug;
use tokio::runtime::current_thread;
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

impl<D> Event<D>
where
    D: Events,
{
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
pub trait Aggregator<E: Events, A: Clone, Q: StoreQuery>: Copy + Clone + Debug + Default {
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
pub trait Store<'a, E: Events, Q: StoreQuery, S: StoreAdapter<E, Q>, C, EM: EmitterAdapter> {
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self;

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<T, A>(&self, query: A) -> Result<T, String>
    where
        T: Aggregator<E, A, Q> + Serialize + for<'de> Deserialize<'de> + PartialEq,
        A: Clone;

    /// Save an event to the store with optional context
    fn save(&self, event: Event<E>) -> Result<(), String>;
}

/// Main event store
pub struct EventStore<S, C, EM> {
    store: S,
    cache: C,
    emitter: EM,
}

impl<'a, E, Q, S, C, EM> Store<'a, E, Q, S, C, EM> for EventStore<S, C, EM>
where
    E: Events + Sync,
    Q: StoreQuery,
    S: StoreAdapter<E, Q>,
    C: CacheAdapter<Q>,
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
    fn aggregate<T, A>(&self, query_args: A) -> Result<T, String>
    where
        T: Aggregator<E, A, Q> + Serialize + for<'de> Deserialize<'de> + PartialEq,
        A: Clone,
    {
        let q = T::query(query_args.clone());
        let initial_state: Option<CacheResult<T>> = self.cache.get(&q);

        self.store.aggregate(query_args, initial_state).map(|agg| {
            if let Some((last_cache, _)) = initial_state {
                // Only update cache if aggregation result has changed
                if agg != last_cache {
                    self.cache.insert(&q, agg);
                }
            } else {
                // If there is no existing cache item, insert one
                self.cache.insert(&q, agg);
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
}
