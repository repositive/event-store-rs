extern crate serde;

use serde::{de::DeserializeOwned, Serialize};

/// Trait to be implemented by the enum of all domain events
///
/// Note (TODO: fix): For enums, the struct name and variant name **MUST** match, or deriving
/// `EventData` won't work correctly.
pub trait EventData: Serialize + DeserializeOwned {
    fn event_namespace_and_type(&self) -> &'static str;

    fn event_namespace(&self) -> &'static str;

    fn event_type(&self) -> &'static str;
}
