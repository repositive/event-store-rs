//! Postgres-backed event store

use super::{Aggregator, Events, Store, StoreQuery};
use fallible_iterator::FallibleIterator;
use postgres::types::ToSql;
use postgres::Connection;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{from_value, Value as JsonValue};
use std::marker::PhantomData;

/// Representation of a Postgres query and args
pub struct PgQuery<'a> {
    /// Query string with placeholders
    query: &'a str,

    /// Arguments to use for the query
    args: Vec<Box<ToSql>>,
}

impl<'a> StoreQuery for PgQuery<'a> {}

impl<'a> PgQuery<'a> {
    /// Create a new query from a query string and arguments
    pub fn new(query: &'a str, args: Vec<Box<ToSql>>) -> Self {
        Self { query, args }
    }
}

/// Postgres-backed event store
pub struct PgStore<E: Events> {
    phantom: PhantomData<E>,
    conn: Connection,
}

impl<'a, E> PgStore<E>
where
    E: Events + Deserialize<'a>,
{
    /// Create a new PgStore from a Postgres DB connection
    pub fn new(conn: Connection) -> Self {
        Self {
            phantom: PhantomData,
            conn,
        }
    }
}

impl<'a, E> Store<E, PgQuery<'a>> for PgStore<E>
where
    E: Events + DeserializeOwned,
{
    fn aggregate<T, A>(&self, query_args: A) -> T
    where
        T: Aggregator<E, A, PgQuery<'a>>,
    {
        let PgQuery { query, args } = T::query(query_args);

        let mut params: Vec<&ToSql> = Vec::new();

        for (i, _arg) in args.iter().enumerate() {
            params.push(&*args[i]);
        }

        let trans = self.conn.transaction().expect("Tranny");
        let stmt = trans.prepare(&query).expect("Prep");

        let results = stmt
            .lazy_query(&trans, &params, 1000)
            .expect("Query")
            .map(|row| {
                let json: JsonValue = row.get("data");
                let evt: E = from_value(json).expect("Decode");

                evt
            }).fold(T::default(), |acc, event| T::apply_event(acc, &event))
            .expect("Fold");

        results
    }
}
