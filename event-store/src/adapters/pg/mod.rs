//! Postgres-backed cache and store adapters

mod cache;
mod store;

pub use self::cache::PgCacheAdapter;
pub use self::store::PgStoreAdapter;
use r2d2::Pool;
use r2d2_postgres::postgres::types::ToSql;
use r2d2_postgres::PostgresConnectionManager;
use sha2::{Digest, Sha256};
use std::str::from_utf8;

use StoreQuery;

type Connection = Pool<PostgresConnectionManager>;

/// Representation of a Postgres query and args
#[derive(Debug)]
pub struct PgQuery<'a> {
    /// Query string with placeholders
    pub query: &'a str,

    /// Arguments to use for the query
    pub args: Vec<Box<ToSql + Send + Sync>>,
}

impl<'a> StoreQuery for PgQuery<'a> {
    fn unique_id(&self) -> String {
        let hash = Sha256::digest(format!("{:?}:[{}]", self.args, self.query).as_bytes());
        String::from(from_utf8(hash.as_slice()).unwrap())
    }
}

impl<'a> PgQuery<'a> {
    /// Create a new query from a query string and arguments
    pub fn new(query: &'a str, args: Vec<Box<ToSql + Send + Sync>>) -> Self {
        Self { query, args }
    }
}
