use super::{CacheAdapter, CacheResult};
use chrono::DateTime;
use chrono::Utc;
use futures::future::ok as FutOk;
use redis::{Client, Commands, Connection};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_str, to_string};
use std::io;
use utils::BoxedFuture;

#[derive(Serialize, Deserialize)]
struct RedisCacheItem<D> {
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
    pub fn new(client: Client) -> Self {
        let conn = client
            .get_connection()
            .expect("Could not get Redis connection");

        Self { client, conn }
    }
}

impl CacheAdapter for RedisCacheAdapter {
    fn set<V>(&self, id: &String, data: V) -> BoxedFuture<(), io::Error>
    where
        V: Serialize + Send,
    {
        let time = Utc::now();

        // TODO: Error handling
        let value: String = to_string(&RedisCacheItem { time, data })
            .expect("Failed to serialize Redis cache item");

        self.conn
            .set::<String, String, ()>(id.to_string(), value)
            // .map_err(|_| {
            //     error!("Failed to set cache item");
            //     "Failed to set cache item".into()
            // })
            .unwrap();

        Box::new(FutOk(()))
    }

    // TODO: Error handling
    fn get<T>(&self, key: &String) -> BoxedFuture<Option<CacheResult<T>>, io::Error>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let value: Option<String> = self.conn.get(key).unwrap();

        let res = value.map(|value| {
            let parsed: RedisCacheItem<T> = from_str(&value).unwrap();

            (parsed.data, parsed.time)
        });

        Box::new(FutOk(res))
    }
}
