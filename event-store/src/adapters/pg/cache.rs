//! Cache adapter backed by postgres

use super::Connection;
use adapters::{CacheAdapter, CacheResult};
use chrono::prelude::*;
use futures::future::ok as FutOk;
use serde::{de::DeserializeOwned, Serialize};
// use serde_json::{from_value, to_value};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use utils::BoxedFuture;

/// Postgres cache adapter
#[derive(Clone)]
pub struct PgCacheAdapter {
    conn: Arc<Mutex<Connection>>,
}

impl PgCacheAdapter {
    /// Create a new PgStore from a Postgres DB connection
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl CacheAdapter for PgCacheAdapter {
    fn insert<V>(&self, key: String, value: V) -> BoxedFuture<(), String>
    where
        V: Serialize,
    {
        Box::new(FutOk(()))
        // let args_hash = Sha256::digest(format!("{:?}:[{}]", key.args, key.query).as_bytes());

        // self.conn
        //     .get()
        //     .expect("Could not get PG connection")
        //     .execute(
        //         r#"INSERT INTO aggregate_cache (id, data, time)
        //         VALUES ($1, $2, NOW())
        //         ON CONFLICT (id)
        //         DO UPDATE SET data = EXCLUDED.data, time = now() RETURNING data"#,
        //         &[&args_hash.as_slice(), &to_value(value).expect("To value")],
        //     ).expect("Cache");
    }

    fn get<'a, T: Sync + Send + DeserializeOwned + 'a>(
        &self,
        key: String,
    ) -> BoxedFuture<'a, Option<CacheResult<T>>, String> {
        Box::new(FutOk(None))
        // let args_hash = Sha256::digest(format!("{:?}:[{}]", key.args, key.query).as_bytes());

        // let rows = self
        //     .conn
        //     .get()
        //     .expect("Could not get PG connection")
        //     .query(
        //         "SELECT data, time FROM aggregate_cache WHERE id = $1 LIMIT 1",
        //         &[&args_hash.as_slice()],
        //     ).expect("Ret");

        // // `rows.get()` panics if index is out of bounds, hence this check
        // if rows.len() != 1 {
        //     None
        // } else {
        //     let row = rows.get(0);

        //     let time: NaiveDateTime = row.get(1);

        //     let utc: DateTime<Utc> = DateTime::from_utc(time, Utc);

        //     Some((
        //         from_value(row.get(0)).map(|decoded: T| decoded).unwrap(),
        //         utc,
        //     ))
        // }
    }
}
