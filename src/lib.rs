//! Event store module for event-driven applications

#![deny(missing_docs)]

mod event;
mod eventstore;

pub use event::Event;
pub use eventstore::{EventStore, PostgresEventStore};
