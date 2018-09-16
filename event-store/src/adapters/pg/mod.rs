//! Postgres-backed cache and store adapters

mod cache;
mod store;

pub use self::cache::PgCacheAdapter;
pub use self::store::PgStoreAdapter;
use bb8_postgres::tokio_postgres::types::ToSql;
use bb8_postgres::tokio_postgres::Connection;

use StoreQuery;

/// Representation of a Postgres query and args
pub struct PgQuery<'a> {
    /// Query string with placeholders
    pub query: &'a str,
}

impl<'a> StoreQuery<'a, &'a str, Vec<Box<ToSql + Send + Sync>>> for PgQuery<'a> {
    fn get_query(&self) -> &'a str {
        self.query
    }
}

impl<'a> PgQuery<'a> {
    /// Create a new query from a query string and arguments
    pub fn new(query: &'a str) -> Self {
        Self { query }
    }
}
