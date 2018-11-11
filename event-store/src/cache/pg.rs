//! Cache adapter backed by postgres

use super::CacheResult;
use chrono::prelude::*;
use postgres::stmt::Statement;
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value};

#[derive(Debug, Clone)]
pub struct PgCacheAdapter<'a> {
    set_stmt: &'a Statement<'a>,
    get_stmt: &'a Statement<'a>,
}

impl<'a> PgCacheAdapter<'a> {
    pub fn new(set_stmt: &'a Statement<'a>, get_stmt: &'a Statement<'a>) -> Self {
        Self { set_stmt, get_stmt }
    }

    pub fn prepare_statements(
        conn: &'a PooledConnection<PostgresConnectionManager>,
    ) -> (Statement<'a>, Statement<'a>) {
        (
            conn.prepare_cached(
                r#"INSERT INTO aggregate_cache (id, data, time)
            VALUES ($1, $2, NOW())
            ON CONFLICT (id)
            DO UPDATE SET data = EXCLUDED.data, time = now() RETURNING data"#,
            )
            .unwrap(),
            conn.prepare_cached(r#"SELECT data, time FROM aggregate_cache WHERE id = $1 LIMIT 1"#)
                .unwrap(),
        )
    }

    pub fn set<V: Serialize>(&self, key: String, value: V) -> Result<(), String> {
        self.set_stmt
            .execute(&[&key, &to_value(value).expect("To value")])
            .map(|_| ())
            .map_err(|_| "Failed to set cache item".into())
    }

    pub fn get<T>(&self, key: String) -> Result<Option<CacheResult<T>>, String>
    where
        T: DeserializeOwned,
    {
        let rows = self
            .get_stmt
            .query(&[&key])
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
