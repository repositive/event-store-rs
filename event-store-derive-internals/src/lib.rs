extern crate serde;

use serde::{de::DeserializeOwned, Serialize};

/// Trait to be implemented by the enum of all domain events. Must also implement `serde::Serialize`
///
/// ```rust
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate event_store;
/// # use event_store::EventData;
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
/// impl EventData for DomainEvents {}
///
/// fn main() {}
/// ```
pub trait EventData: Serialize + DeserializeOwned {
    fn namespaced_type(&self) -> &'static str;
}
