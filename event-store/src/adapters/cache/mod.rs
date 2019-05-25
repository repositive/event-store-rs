use chrono::prelude::*;

mod pg;

// TODO: Rename this. `Result` implies an error condition, but it's not. Maybe `CacheItem`? Idk.
/// Result of a cache search
pub type CacheResult<T> = (T, DateTime<Utc>);

pub use self::pg::PgCacheAdapter;
