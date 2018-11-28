use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io;
use utils::BoxedFuture;

mod pg;
mod redis;

pub use self::pg::PgCacheAdapter;
pub use self::redis::RedisCacheAdapter;

/// Result of a cache search
pub type CacheResult<T> = (T, DateTime<Utc>);

/// Caching backend
pub trait CacheAdapter: Clone + Send + 'static {
    /// Insert an item into the cache
    fn set<V>(&self, key: &String, value: V) -> BoxedFuture<(), io::Error>
    where
        V: Serialize + Send;

    /// Retrieve an item from the cache
    fn get<T>(&self, key: &String) -> BoxedFuture<Option<CacheResult<T>>, io::Error>
    where
        T: DeserializeOwned + Send + 'static;
}
