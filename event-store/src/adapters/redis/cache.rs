use super::RedisCacheItem;
use adapters::{CacheAdapter, CacheResult};
use chrono::Utc;
use redis::{Client, Commands, Connection};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_str, to_string};

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
    fn set<V>(&self, id: String, data: V) -> Result<(), String>
    where
        V: Serialize + Send,
    {
        let time = Utc::now();

        // TODO: Error handling
        let value: String = to_string(&RedisCacheItem { time, data })
            .expect("Failed to serialize Redis cache item");

        self.conn.set(id, value).map_err(|err| {
            error!("Failed to set cache item: {:?}", err);

            "Failed to set cache item".into()
        })
    }

    // TODO: Error handling
    fn get<T>(&self, key: String) -> Result<Option<CacheResult<T>>, String>
    where
        T: DeserializeOwned + Send,
    {
        let value: Option<String> = self.conn.get(key).unwrap();

        Ok(value.map(|value| {
            let parsed: RedisCacheItem<T> = from_str(&value).unwrap();

            (parsed.data, parsed.time)
        }))
    }
}
