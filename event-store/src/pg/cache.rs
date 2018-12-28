use chrono::prelude::*;
use log::{debug, trace};
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
pub async fn pg_cache_save<V>(
    conn: PooledConnection<PostgresConnectionManager>,
    key: String,
    value: &V,
) -> Result<(), io::Error>
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
    .map(|_| ())
    .map_err(|e| e.into())
    // .map(|_| future::ok(()))
    // .unwrap_or_else(|_| future::err(io::Error::new(io::ErrorKind::Other, "Could read cache")))
}

/// Read a cache item
pub async fn pg_cache_read<T>(
    conn: PooledConnection<PostgresConnectionManager>,
    key: String,
) -> Result<Option<CacheResult<T>>, io::Error>
where
    T: DeserializeOwned + Debug,
{
    trace!("Cache read key {}", key);

    conn.query(
        "SELECT data, time FROM aggregate_cache WHERE id = $1 LIMIT 1",
        &[&key],
    )
    .map(|rows| {
        // `rows.get()` panics if index is out of bounds, hence this check
        let res = if rows.len() != 1 {
            None
        } else {
            let row = rows.get(0);

            let time: NaiveDateTime = row.get(1);

            let utc: DateTime<Utc> = DateTime::from_utc(time, Utc);

            Some((
                from_value(row.get(0))
                    .map(|decoded: T| decoded)
                    .expect("Cant decode the cached entity"),
                utc,
            ))
        };

        trace!("Cache read result {:?}", res);

        // future::ok((conn, res))

        res
    })
    .map_err(|e| e.into())
    // .unwrap_or_else(|_| future::err(io::Error::new(io::ErrorKind::Other, "Could read cache")))
}
