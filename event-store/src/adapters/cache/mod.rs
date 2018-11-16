use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;

mod pg;
mod redis;

pub use self::pg::PgCacheAdapter;
pub use self::redis::RedisCacheAdapter;

/// Result of a cache search
pub type CacheResult<T> = (T, DateTime<Utc>);

/// Caching backend
pub trait CacheAdapter: Clone + Send + 'static {
    /// Insert an item into the cache
    fn set<V>(&self, key: String, value: V) -> Result<(), String>
    where
        V: Serialize + Send;

    /// Retrieve an item from the cache
    fn get<T>(&self, key: String) -> Result<Option<CacheResult<T>>, String>
    where
        T: DeserializeOwned + Send;
}
