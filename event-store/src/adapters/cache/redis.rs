// TODO: Remove when Redis cache is usable
#![allow(unused)]

use super::CacheResult;
use chrono::{DateTime, Utc};
use log::error;
use redis::{Client, Commands, Connection};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_str, to_string};
use std::io;

#[derive(Serialize, Deserialize)]
pub struct RedisCacheItem<D> {
    data: D,
    time: DateTime<Utc>,
}

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
    pub async fn new(client: Client) -> Result<Self, io::Error> {
        let conn = client
            .get_connection()
            .expect("Could not get Redis connection");

        Ok(Self { client, conn })
    }

    pub async fn set<'a, V>(&'a self, id: String, data: V) -> Result<(), String>
    where
        V: Serialize + 'a,
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
    pub async fn get<T>(&self, key: String) -> Result<Option<CacheResult<T>>, String>
    where
        T: DeserializeOwned,
    {
        let value: Option<String> = self.conn.get(key).unwrap();

        Ok(value.map(|value| {
            let parsed: RedisCacheItem<T> = from_str(&value).unwrap();

            (parsed.data, parsed.time)
        }))
    }
}
