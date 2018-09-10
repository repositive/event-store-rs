//! Event store prelude

pub use adapters::{CacheAdapter, CacheResult, EmitterAdapter, StoreAdapter};
pub use aggregator::Aggregator;
pub use event_store_derive_internals::{EventData, Events};
pub use store::Store;
pub use store_query::StoreQuery;
