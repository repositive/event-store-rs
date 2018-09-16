use event_store_derive_internals::Events;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use store_query::StoreQuery;

/// Aggregator trait
///
/// This takes three type items:
///
/// * `E: EventData` – The enum of domain events that items from the backing store will be parsed to
/// * `A` – The type of the query args to use when querying the backing store. Can be as simple as
/// a `String`, but using a `struct` is recommended for readability
/// * `Q: StoreQuery` – The query object to pass to the backing store. It should be built from `A`
///
/// ### Aggregation from `E: Events` into `Self`
///
/// Aggregation logic is handled by `apply_event`. Every event in the resultset is passed to this
/// function. It should return a modified copy of `Self` if a field has changed, or just `self` if
/// the event does not change any state. For example:
///
/// ```rust
/// # extern crate serde;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate event_store;
/// # #[macro_use]
/// # extern crate event_store_derive;
/// use event_store::prelude::*;
/// use event_store::Event;
///
/// // An example event in this domain
/// #[derive(EventData, Debug)]
/// #[event_store(namespace = "some_namespace")]
/// struct NameChanged {
///     name: String,
/// }
///
/// // Another example event in this domain
/// #[derive(EventData, Debug)]
/// #[event_store(namespace = "some_namespace")]
/// struct EmailChanged {
///     email: String,
/// }
///
/// // We want to ignore this event; the password should never be output!
/// #[derive(EventData, Debug)]
/// #[event_store(namespace = "some_namespace")]
/// struct PasswordChanged {
///     password: String,
/// }
///
/// // Enum of all events for this domain
/// #[derive(Events)]
/// enum UsersEvents {
///     NameChanged(Event<NameChanged>),
///     EmailChanged(Event<EmailChanged>),
///     PasswordChanged(Event<PasswordChanged>),
/// }
///
/// // The domain entity we want to aggregate to
/// #[derive(Clone, Debug, PartialEq)]
/// struct UserDetails {
///     name: String,
///     email: String,
/// }
///
/// // Initial state for user detail aggregation
/// impl Default for UserDetails {
///     fn default() -> Self {
///         Self {
///             name: "".into(),
///             email: "".into(),
///         }
///     }
/// }
/// #
/// # struct DummyQuery;
/// # impl StoreQuery for DummyQuery {}
///
/// impl Aggregator<UsersEvents, String, DummyQuery> for UserDetails {
///     fn apply_event(acc: Self, event: &UsersEvents) -> Self {
///         match event {
///             UsersEvents::NameChanged(e) => Self {
///                 name: e.data.name.clone(),
///                 ..acc
///             },
///             UsersEvents::EmailChanged(e) => Self {
///                 email: e.data.email.clone(),
///                 ..acc
///             },
///             UsersEvents::PasswordChanged(_) => acc,
///         }
///     }
///
///     // ...
/// #
/// #     fn query(_: String) -> DummyQuery {
/// #         DummyQuery
/// #     }
/// }
///
/// fn main() {
/// #     let example_events = vec![
/// #         UsersEvents::NameChanged(Event::from_data(NameChanged { name: String::from("Repositive User") })),
/// #         UsersEvents::EmailChanged(Event::from_data(EmailChanged { email: String::from("bobby@beans.com") })),
/// #         UsersEvents::PasswordChanged(Event::from_data(PasswordChanged { password: String::from("hunter2") })),
/// #     ];
/// #
///     // Example aggregation without a store. You would normally call `store.aggregate`
///     let user: UserDetails = example_events
///         .iter()
///         .fold(UserDetails::default(), UserDetails::apply_event);
///
///     assert_eq!(
///         user,
///         UserDetails { name: "Repositive User".into(), email: "bobby@beans.com".into() }
///     );
/// }
/// ```
///
/// ### Initial state
///
/// `Aggregator` has a trait bound on `Default`. `Default` should be the initial state
/// of the entity before the events are reduced onto it. For example:
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
pub trait Aggregator<'a, E: Events, A, Q: StoreQuery<'a, A>>:
    Send + Sync + Clone + Debug + Default + Serialize + DeserializeOwned + PartialEq + 'a
{
    /// Apply an event `E` to `acc`, returning a copy of `Self` with updated fields. Can also just
    /// return `acc` if nothing has changed.
    fn apply_event(acc: Self, event: &E) -> Self;

    /// Produce a query object from some query arguments
    fn query(args: A) -> Q;
}
