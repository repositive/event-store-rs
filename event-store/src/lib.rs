//! Event store

#![deny(missing_docs)]
// enable the await! macro, async support, and the new std::Futures api.
#![feature(await_macro, async_await, futures_api)]
// only needed to manually implement a std future:
#![feature(arbitrary_self_types)]

#[macro_use]
extern crate serde_derive;

mod aggregator;
mod event;
mod event_context;
mod event_handler;
mod store;
mod store_query;
mod subscribable_store;

pub mod adapters;
#[doc(hidden)]
pub mod internals;
pub mod prelude;

pub use crate::aggregator::Aggregator;
pub use crate::event::Event;
pub use crate::event_context::EventContext;
pub use crate::event_handler::EventHandler;
pub use crate::store::Store;
pub use crate::store_query::StoreQuery;
pub use crate::subscribable_store::SubscribableStore;
pub use event_store_derive_internals::{EventData, Events};
