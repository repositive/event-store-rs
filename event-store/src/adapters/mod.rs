//! Adapters for use with event store integrations
//!
//! A store will require a storage backend, cache backend and an event emitter instance for
//! integration with other event-driven domains. Use the adapters exposed by this module to suit
//! your application and architecture.

mod cache;
mod emitter;
mod store;

pub use self::cache::{CacheAdapter, CacheResult, PgCacheAdapter, RedisCacheAdapter};
pub use self::emitter::{
    AMQPEmitterAdapter, AMQPEmitterOptions, EmitterAdapter, StubEmitterAdapter,
};
pub use self::store::{PgQuery, PgStoreAdapter, StoreAdapter, StoreQuery};
