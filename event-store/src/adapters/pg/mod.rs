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
    /// Arguments to use for the query
    pub args: Vec<Box<ToSql + 'a>>,
}

impl<'a> StoreQuery<'a> for PgQuery<'a> {}

impl<'a> PgQuery<'a> {
    /// Create a new query from a query string and arguments
    pub fn new(query: &'a str, args: Vec<Box<ToSql + 'a>>) -> Self {
        Self { query, args }
    }
}
