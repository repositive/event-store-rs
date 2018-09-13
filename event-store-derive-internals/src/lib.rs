extern crate serde;

use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

/// Trait to be implemented by all domain events
pub trait EventData: Clone + Sync + Debug + Serialize + DeserializeOwned {
    fn event_namespace_and_type() -> &'static str;

    fn event_namespace() -> &'static str;

    fn event_type() -> &'static str;
}

/// Trait implemented on the events enum
pub trait Events: Serialize + DeserializeOwned {}
