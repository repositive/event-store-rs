use chrono::DateTime;
use chrono::Utc;

pub mod pg;

pub type CacheResult<T> = (T, DateTime<Utc>);
