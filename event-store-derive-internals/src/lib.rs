extern crate serde;

use serde::{de::DeserializeOwned, Serialize};

/// Trait to be implemented by all domain events
pub trait EventData: Serialize + DeserializeOwned {
    fn event_namespace_and_type() -> &'static str;

    fn event_namespace() -> &'static str;

    fn event_type() -> &'static str;
}

/// Trait implemented on the events enum
pub trait Events: Serialize + DeserializeOwned {
    fn event_namespace_and_type(&self) -> &'static str;

    fn event_namespace(&self) -> &'static str;

    fn event_type(&self) -> &'static str;
}
