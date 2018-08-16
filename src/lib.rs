//! Event store for working with event-store-driven applications

#![deny(missing_docs)]

extern crate fallible_iterator;
extern crate postgres;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate sha2;

pub mod pg;
pub mod testhelpers;

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

/// Trait to be implemented by all domain events
pub trait Event {}

/// Trait to be implemented by the enum of all domain events
///
/// ```rust
/// # use event_store_rs::Events;
/// struct EventA;
/// struct EventB;
///
/// enum DomainEvents {
///     A(EventA),
///     B(EventB),
/// }
///
/// impl Events for DomainEvents {}
/// ```
pub trait Events {}

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
pub trait Store<E: Events, Q: StoreQuery> {
    /// Query the backing store and return an entity `T`, reduced from queried events
    fn aggregate<T, A>(&self, query: A) -> T
    where
        E: Events,
        A: Clone,
        T: Aggregator<E, A, Q> + Serialize + DeserializeOwned;
}
