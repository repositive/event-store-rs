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

#[derive(Clone)]
pub struct PgCacheAdapter {
    conn: Pool<PostgresConnectionManager>,
}

impl PgCacheAdapter {
    pub async fn new(conn: Pool<PostgresConnectionManager>) -> Result<Self, io::Error> {
        Ok(Self { conn })
    }

    pub async fn read<T>(&self, key: String) -> Result<Option<CacheResult<T>>, io::Error>
    where
        T: DeserializeOwned + Debug,
    {
        trace!("Cache read key {}", key);

        self.conn
            .get()
            .unwrap()
            .query(
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

                res
            })
            .map_err(|e| e.into())
    }

    pub async fn save<'a, V>(&'a self, key: String, value: &'a V) -> Result<(), io::Error>
    where
        V: Serialize + Debug,
    {
        debug!("Cache aggregate result under key {}: {:?}", key, value);

        self.conn
            .get()
            .unwrap()
            .execute(
                r#"INSERT INTO aggregate_cache (id, data, time)
                    VALUES ($1, $2, NOW())
                    ON CONFLICT (id)
                    DO UPDATE SET data = EXCLUDED.data, time = now() RETURNING data"#,
                &[&key, &to_value(value).expect("To value")],
            )
            .map(|_| ())
            .map_err(|e| e.into())
    }
}
