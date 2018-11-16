//! Cache adapter backed by postgres

use super::{CacheAdapter, CacheResult};
use chrono::prelude::*;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value};

/// Postgres cache adapter
#[derive(Clone)]
pub struct PgCacheAdapter {
    conn: Pool<PostgresConnectionManager>,
}

impl PgCacheAdapter {
    /// Create a new PgStore from a Postgres DB connection
    pub fn new(conn: Pool<PostgresConnectionManager>) -> Self {
        Self { conn }
    }
}

impl CacheAdapter for PgCacheAdapter {
    fn set<V: Serialize + Send>(&self, key: String, value: V) -> Result<(), String> {
        self.conn
            .get()
            .expect("Could not get PG connection")
            .execute(
                r#"INSERT INTO aggregate_cache (id, data, time)
                VALUES ($1, $2, NOW())
                ON CONFLICT (id)
                DO UPDATE SET data = EXCLUDED.data, time = now() RETURNING data"#,
                &[&key, &to_value(value).expect("To value")],
            )
            .map(|_| ())
            .map_err(|_| "Failed to set cache item".into())
    }

    fn get<T>(&self, key: String) -> Result<Option<CacheResult<T>>, String>
    where
        T: DeserializeOwned + Send,
    {
        let rows = self
            .conn
            .get()
            .expect("Could not get PG connection")
            .query(
                "SELECT data, time FROM aggregate_cache WHERE id = $1 LIMIT 1",
                &[&key],
            )
            .map_err(|e| format!("Retrieve cache: {:?}", e))?;

        // `rows.get()` panics if index is out of bounds, hence this check
        if rows.len() != 1 {
            Ok(None)
        } else {
            let row = rows.get(0);

            let time: NaiveDateTime = row.get(1);

            let utc: DateTime<Utc> = DateTime::from_utc(time, Utc);

            Ok(Some((
                from_value(row.get(0))
                    .map(|decoded: T| decoded)
                    .expect("Cant decode the cached entity"),
                utc,
            )))
        }
    }
}
