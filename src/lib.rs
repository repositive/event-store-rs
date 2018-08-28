//! Event store for working with event-store-driven applications

#![deny(missing_docs)]

extern crate fallible_iterator;
extern crate postgres;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate sha2;
extern crate uuid;

pub mod adapters;
pub mod testhelpers;

use adapters::{CacheAdapter, CacheResult, EmitterAdapter, StoreAdapter};
use chrono::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fmt::Debug;

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

/// Trait to be implemented by all domain events
pub trait Event {}

/// Trait to be implemented by the enum of all domain events. Must also implement `serde::Serialize`
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate event_store;
/// # use event_store::Events;
/// #[derive(Serialize, Deserialize)]
/// struct EventA;
///
/// #[derive(Serialize, Deserialize)]
/// struct EventB;
///
/// #[derive(Serialize, Deserialize)]
/// enum DomainEvents {
///     A(EventA),
///     B(EventB),
/// }
///
/// impl Events for DomainEvents {}
///
/// fn main() {}
/// ```
pub trait Events: Serialize + DeserializeOwned {}

/// A query to be passed to the store
///
/// This trait must be implemented for whichever type you want to pass to a particular store. See
/// impls below for examples.
pub trait StoreQuery {}

/// Aggregator trait
///
/// This takes three type items:
///
/// * `E: Events` – The enum of domain events that rows from the backing store will be parsed to
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
    fn apply_event(acc: Self, event: &E) -> Self;

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
pub trait Store<'a, E: Events, Q: StoreQuery, S: StoreAdapter<E, Q>, C, EM> {
    /// Create a new event store
    fn new(store: S, cache: C, emitter: EM) -> Self;

    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<T, A>(&self, query: A) -> Result<T, String>
    where
        T: Aggregator<E, A, Q> + Serialize + for<'de> Deserialize<'de> + PartialEq,
        A: Clone;

    /// Save an event to the store with optional context
    fn save<CO>(&self, event: E, subject: Option<CO>) -> Result<(), String>
    where
        CO: Serialize;
}

/// Main event store
pub struct EventStore<S, C, EM> {
    store: S,
    cache: C,
    emitter: EM,
}

impl<'a, E, Q, S, C, EM> Store<'a, E, Q, S, C, EM> for EventStore<S, C, EM>
where
    E: Events,
    Q: StoreQuery,
    S: StoreAdapter<E, Q>,
    C: CacheAdapter<Q>,
    EM: EmitterAdapter<E>,
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
    fn save<CO>(&self, event: E, subject: Option<CO>) -> Result<(), String>
    where
        CO: Serialize,
    {
        self.store.save(&event, subject).expect("Save");

        self.emitter.emit(&event);

        Ok(())
    }
}
