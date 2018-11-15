use adapters::{CacheAdapter, CacheResult};
use chrono::{DateTime, NaiveDateTime, Utc};
use redis::{Client, Connection};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_value, to_value};

/// Redis cache adapter
pub struct RedisCacheAdapter {
    client: Client,
    conn: Connection,
}

impl Clone for RedisCacheAdapter {
    fn clone(&self) -> Self {
        let client = self.client.clone();
        let conn = client
            .get_connection()
            .expect("Could not clone Redis cache adapter");

        Self { client, conn }
    }
}

impl RedisCacheAdapter {
    /// Create a new Redis-backed cache from a Redis client handle
    ///
    /// A connection to Redis is created from the client each time the adapter is created **or
    /// cloned**. It should be cloned as little as possible.
    pub fn new(client: Client) -> Self {
        let conn = client
            .get_connection()
            .expect("Could not get Redis connection");

        Self { client, conn }
    }
}

impl CacheAdapter for RedisCacheAdapter {
    fn set<V>(&self, key: String, value: V) -> Result<(), String>
    where
        V: Serialize + Send,
    {
        // TODO: Implement
        Ok(())
    }

    fn get<T>(&self, key: String) -> Result<Option<CacheResult<T>>, String>
    where
        T: DeserializeOwned + Send,
    {
        // TODO: Implement
        Ok(None)
    }
}
