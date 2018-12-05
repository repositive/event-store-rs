use chrono::prelude::*;
use futures::future;
use futures::Future;
use r2d2::{self, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::from_value;
use serde_json::to_value;
use std::fmt::Debug;
use std::io;

/// Result of a cache search
pub type CacheResult<T> = (T, DateTime<Utc>);

/// Save an event into PG
pub fn pg_cache_save<V>(
    conn: PooledConnection<PostgresConnectionManager>,
    key: String,
    value: &V,
) -> impl Future<Item = (), Error = io::Error>
where
    V: Serialize + Debug,
{
    debug!("Cache aggregate result under key {}: {:?}", key, value);

    conn.execute(
        r#"INSERT INTO aggregate_cache (id, data, time)
                VALUES ($1, $2, NOW())
                ON CONFLICT (id)
                DO UPDATE SET data = EXCLUDED.data, time = now() RETURNING data"#,
        &[&key, &to_value(value).expect("To value")],
    )
    .map(|_| future::ok(()))
    .unwrap_or_else(|_| future::err(io::Error::new(io::ErrorKind::Other, "Could read cache")))
}

/// Read a cache item
pub fn pg_cache_read<T>(
    conn: PooledConnection<PostgresConnectionManager>,
    key: String,
) -> impl Future<Item = Option<CacheResult<T>>, Error = io::Error>
where
    T: DeserializeOwned,
{
    conn.query(
        "SELECT data, time FROM aggregate_cache WHERE id = $1 LIMIT 1",
        &[&key],
    )
    .map(|rows| {
        // `rows.get()` panics if index is out of bounds, hence this check
        if rows.len() != 1 {
            future::ok(None)
        } else {
            let row = rows.get(0);

            let time: NaiveDateTime = row.get(1);

            let utc: DateTime<Utc> = DateTime::from_utc(time, Utc);

            future::ok(Some((
                from_value(row.get(0))
                    .map(|decoded: T| decoded)
                    .expect("Cant decode the cached entity"),
                utc,
            )))
        }
    })
    .unwrap_or_else(|_| future::err(io::Error::new(io::ErrorKind::Other, "Could read cache")))
}
