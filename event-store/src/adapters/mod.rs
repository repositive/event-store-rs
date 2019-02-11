//! Backing store adapters for event storage, caching and subscriptions

mod cache;
mod emitter;
mod store;

pub use self::cache::{CacheResult, PgCacheAdapter};
pub use self::emitter::AmqpEmitterAdapter;
pub use self::store::{LastHandledEvent, PgQuery, PgStoreAdapter, SaveResult, SaveStatus};
