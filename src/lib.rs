//! Event store module for event-driven applications

#![deny(missing_docs)]

#[feature(postgres_support)]
extern crate r2d2;
#[feature(postgres_support)]
extern crate r2d2_postgres;

mod event;
mod eventstore;

pub use event::Event;
pub use eventstore::EventStore;
#[feature(postgres_support)]
pub use eventstore::PostgresEventStore;
