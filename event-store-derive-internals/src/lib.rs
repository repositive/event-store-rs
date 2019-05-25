//! Event store internal types

#![deny(missing_docs)]

/// Trait implemented by any set of entity creation events
pub trait EventStoreCreateEvents {}

/// Trait implemented by any set of entity update events
pub trait EventStoreUpdateEvents {}
