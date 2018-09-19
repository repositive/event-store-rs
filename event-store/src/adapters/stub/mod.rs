//! Stub implementations of things

mod cache;
mod emitter;
mod store;

use store_query::StoreQuery;

/// Stub query that doesn't do anything
pub struct StubQuery {}

impl StoreQuery for StubQuery {
    fn unique_id(&self) -> String {
        String::new()
    }
}

pub use self::cache::StubCacheAdapter;
pub use self::emitter::StubEmitterAdapter;
pub use self::store::StubStoreAdapter;
