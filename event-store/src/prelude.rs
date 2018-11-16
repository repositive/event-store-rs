//! Event store prelude

pub use adapters::{CacheAdapter, EmitterAdapter, StoreAdapter, StoreQuery};
pub use aggregator::Aggregator;
pub use event_store_derive_internals::{EventData, Events};
pub use store::Store;
