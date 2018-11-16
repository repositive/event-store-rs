mod cache;

pub use self::cache::RedisCacheAdapter;

use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
struct RedisCacheItem<D> {
    data: D,
    time: DateTime<Utc>,
}
