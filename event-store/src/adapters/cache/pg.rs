use super::CacheResult;
use chrono::prelude::*;
use log::{debug, trace};
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::from_value;
use serde_json::to_value;
use std::fmt::Debug;
use std::io;

const INIT_QUERIES: &'static str = r#"
-- Create UUID extension just in case
create extension if not exists "uuid-ossp";

-- Create cache table if it doesn't already exist
create table if not exists aggregate_cache(
    id varchar(64) not null,
    data jsonb not null,
    time timestamp with time zone,
    primary key(id)
);

create index if not exists cache_time on aggregate_cache (time desc);
"#;

/// Postgres-backed cache adapter
#[derive(Clone)]
pub struct PgCacheAdapter {
    conn: Pool<PostgresConnectionManager>,
}

impl PgCacheAdapter {
    /// Create a new PG-backed cache adapter instance
    ///
    /// This will attempt to create the cache table if it does not already exist
    pub async fn new(conn: Pool<PostgresConnectionManager>) -> Result<Self, io::Error> {
        conn.get()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
            .batch_execute(INIT_QUERIES)?;

        Ok(Self { conn })
    }

    /// Read an item from the cache by key, parsing to type `T`
    pub async fn read<'a, T>(&'a self, key: &'a str) -> Result<Option<CacheResult<T>>, io::Error>
    where
        T: DeserializeOwned + Debug,
    {
        trace!("Cache read key {}", key);

        self.conn
            .get()
            .unwrap()
            .query(
                "select data, time from aggregate_cache where id = $1 limit 1",
                &[&key],
            )
            .map(|rows| {
                // `rows.get()` panics if index is out of bounds, hence this check
                let res = if rows.len() != 1 {
                    None
                } else {
                    let row = rows.get(0);
                    let utc: DateTime<Utc> = row.get(1);

                    Some((
                        from_value(row.get(0))
                            .map(|decoded: T| decoded)
                            .expect("Cant decode the cached entity"),
                        utc,
                    ))
                };

                trace!("Cache read result {:?}", res);

                res
            })
            .map_err(|e| e.into())
    }

    /// Save an event into the cache
    pub async fn save<'a, V>(&'a self, key: &'a str, value: &'a V) -> Result<(), io::Error>
    where
        V: Serialize + Debug,
    {
        debug!("Cache aggregate result under key {}: {:?}", key, value);

        self.conn
            .get()
            .unwrap()
            .execute(
                r#"insert into aggregate_cache (id, data, time)
                    values ($1, $2, now())
                    on conflict (id)
                    do update set data = excluded.data, time = now() returning data"#,
                &[&key, &to_value(value).expect("To value")],
            )
            .map(|_| ())
            .map_err(|e| e.into())
    }
}
