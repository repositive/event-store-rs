//! Cache adapter backed by postgres

use super::Connection;
use adapters::{CacheAdapter, CacheResult};
use chrono::prelude::*;
use futures::future::{lazy as FutLazy, ok as FutOk};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value};
use utils::BoxedFuture;

/// Postgres cache adapter
#[derive(Clone)]
pub struct PgCacheAdapter {
    conn: Connection,
}

impl PgCacheAdapter {
    /// Create a new PgStore from a Postgres DB connection
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

impl CacheAdapter for PgCacheAdapter {
    fn set<'a, V: Serialize + Send + 'a>(
        &self,
        key: String,
        value: V,
    ) -> BoxedFuture<'a, (), String> {
        let conn = self.conn.clone();
        Box::new(FutLazy(move || {
            conn.get()
                .expect("Could not get PG connection")
                .execute(
                    r#"INSERT INTO aggregate_cache (id, data, time)
                VALUES ($1, $2, NOW())
                ON CONFLICT (id)
                DO UPDATE SET data = EXCLUDED.data, time = now() RETURNING data"#,
                    &[&key, &to_value(value).expect("To value")],
                ).expect("Update cache");
            FutOk(())
        }))
    }

    fn get<'a, T>(&self, key: String) -> BoxedFuture<'a, Option<CacheResult<T>>, String>
    where
        T: DeserializeOwned + Send + 'a,
    {
        let conn = self.conn.clone();
        Box::new(FutLazy(move || {
            let rows = conn
                .get()
                .expect("Could not get PG connection")
                .query(
                    "SELECT data, time FROM aggregate_cache WHERE id = $1 LIMIT 1",
                    &[&key],
                ).expect("Retrieve cache");

            // `rows.get()` panics if index is out of bounds, hence this check
            if rows.len() != 1 {
                Box::new(FutOk(None))
            } else {
                let row = rows.get(0);

                let time: NaiveDateTime = row.get(1);

                let utc: DateTime<Utc> = DateTime::from_utc(time, Utc);

                Box::new(FutOk(Some((
                    from_value(row.get(0))
                        .map(|decoded: T| decoded)
                        .expect("Cant decode the cached entity"),
                    utc,
                ))))
            }
        }))
    }
}
