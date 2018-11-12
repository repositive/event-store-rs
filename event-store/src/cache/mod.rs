use chrono::DateTime;
use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

pub mod pg;

pub type CacheResult<T> = (T, DateTime<Utc>);

pub trait CacheAdapter {
    fn set<V>(&self, key: String, value: V) -> Result<(), String>
    where
        V: Serialize + Debug;

    fn get<T>(&self, key: String) -> Result<Option<CacheResult<T>>, String>
    where
        T: DeserializeOwned;
}
