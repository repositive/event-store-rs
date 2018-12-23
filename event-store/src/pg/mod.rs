mod cache;
mod store;

pub use self::cache::*;
pub use self::store::*;
use crate::store_query::StoreQuery;
use postgres::types::ToSql;
use r2d2::{self, Pool};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use sha2::{Digest, Sha256};

/// Representation of a Postgres query and args
#[derive(Debug)]
pub struct PgQuery {
    /// Query string with placeholders
    pub query: String,

    /// Arguments to use for the query
    // TODO: Remove `Sync` (and `Send`?) when we no longer need to use old futures
    pub args: Vec<Box<ToSql + Send + Sync>>,
}

impl PgQuery {
    /// Create a new query from a query string and arguments
    pub fn new(query: &str, args: Vec<Box<ToSql + Send + Sync>>) -> Self {
        Self {
            query: query.into(),
            args,
        }
    }
}

impl StoreQuery for PgQuery {
    fn unique_id(&self) -> String {
        let hash = Sha256::digest(format!("{:?}:[{}]", self.args, self.query).as_bytes());
        hash.iter().fold(String::new(), |mut acc, hex| {
            acc.push_str(&format!("{:X}", hex));
            acc
        })
    }
}

/// Connect to a local Postgres database on port 5430
pub fn pg_connect() -> Pool<PostgresConnectionManager> {
    let manager = PostgresConnectionManager::new(
        "postgres://postgres@localhost:5430/eventstorerust",
        TlsMode::None,
    )
    .unwrap();

    let pool = r2d2::Pool::new(manager).unwrap();

    pool
}
