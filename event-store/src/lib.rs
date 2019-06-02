//! Event store crate

#![deny(missing_docs)]

mod event;
// mod event_store;

pub use crate::event::Event;
pub use event_store_derive::{CreateEvents, UpdateEvents};
pub use event_store_derive_internals::{EventStoreCreateEvents, EventStoreUpdateEvents};

// TODO: Aggregator trait
