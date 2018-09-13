//! Postgres-backed cache and store adapters

mod cache;
mod store;

pub use self::cache::PgCacheAdapter;
pub use self::store::PgStoreAdapter;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use postgres::types::ToSql;

use StoreQuery;

type Connection = Pool<PostgresConnectionManager>;

/// Representation of a Postgres query and args
pub struct PgQuery<'a> {
    /// Query string with placeholders
    pub query: &'a str,

    /// Arguments to use for the query
    pub args: Vec<Box<ToSql + Send + Sync>>,
}

impl<'a> StoreQuery for PgQuery<'a> {}

impl<'a> PgQuery<'a> {
    /// Create a new query from a query string and arguments
    pub fn new(query: &'a str, args: Vec<Box<ToSql + Send + Sync>>) -> Self {
        Self { query, args }
    }
}
