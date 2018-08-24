//! Cache adapter backed by postgres

use adapters::pg::PgQuery;
use adapters::{CacheAdapter, CacheResult};
use chrono::prelude::*;
use postgres::Connection;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_json::from_value;
use serde_json::to_value;
use sha2::{Digest, Sha256};

/// Postgres cache adapter
pub struct PgCacheAdapter {
    conn: Connection,
}

impl PgCacheAdapter {
    /// Create a new PgStore from a Postgres DB connection
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

impl<'a> CacheAdapter<PgQuery<'a>> for PgCacheAdapter {
    fn insert<V>(&self, key: &PgQuery, value: V)
    where
        V: Serialize,
    {
        let args_hash = Sha256::digest(format!("{:?}:[{}]", key.args, key.query).as_bytes());

        self.conn
            .execute(
                r#"INSERT INTO aggregate_cache (id, data, time)
                VALUES ($1, $2, NOW())
                ON CONFLICT (id)
                DO UPDATE SET data = EXCLUDED.data, time = now() RETURNING data"#,
                &[&args_hash.as_slice(), &to_value(value).expect("To value")],
            ).expect("Cache");
    }

    fn get<T>(&self, key: &PgQuery) -> Option<CacheResult<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let args_hash = Sha256::digest(format!("{:?}:[{}]", key.args, key.query).as_bytes());

        let rows = self
            .conn
            .query(
                "SELECT data, time FROM aggregate_cache WHERE id = $1 LIMIT 1",
                &[&args_hash.as_slice()],
            ).expect("Ret");

        // `rows.get()` panics if index is out of bounds, hence this check
        if rows.len() != 1 {
            None
        } else {
            let row = rows.get(0);

            let time: NaiveDateTime = row.get(1);

            let utc: DateTime<Utc> = DateTime::from_utc(time, Utc);

            Some((
                from_value(row.get(0)).map(|decoded: T| decoded).unwrap(),
                utc,
            ))
        }
    }
}
