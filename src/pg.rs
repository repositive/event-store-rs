//! Postgres-backed event store

use super::{Aggregator, Events, Store, StoreQuery};
use fallible_iterator::FallibleIterator;
use postgres::types::ToSql;
use postgres::Connection;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{from_value, to_value, Value as JsonValue};
use sha2::{Digest, Sha256};
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

    fn cache_save<T>(&self, q: &PgQuery, result: &T)
    where
        T: Serialize,
    {
        let args_hash = Sha256::digest(format!("{:?}:[{}]", q.args, q.query).as_bytes());

        self.conn
            .execute(
                r#"INSERT INTO aggregate_cache (id, data, time)
                VALUES ($1, $2, NOW())
                ON CONFLICT (id)
                DO UPDATE SET data = EXCLUDED.data, time = now() RETURNING data"#,
                &[&args_hash.as_slice(), &to_value(result).expect("To value")],
            ).expect("Cache");
    }

    fn cache_find<T>(&self, q: &PgQuery) -> Option<T>
    where
        T: DeserializeOwned + Default,
    {
        let args_hash = Sha256::digest(format!("{:?}:[{}]", q.args, q.query).as_bytes());

        let rows = self
            .conn
            .query(
                "SELECT data FROM aggregate_cache WHERE id = $1 LIMIT 1",
                &[&args_hash.as_slice()],
            ).expect("Ret");

        // `rows.get()` panics if index is out of bounds, hence this check. Should be an Option.
        if rows.len() != 1 {
            None
        } else {
            from_value(rows.get(0).get(0))
                .map(|decoded: T| decoded)
                .ok()
        }
    }
}

impl<'a, E> Store<E, PgQuery<'a>> for PgStore<E>
where
    E: Events + DeserializeOwned + Serialize,
{
    fn aggregate<T, A>(&self, query_args: A) -> T
    where
        T: Aggregator<E, A, PgQuery<'a>> + Serialize + DeserializeOwned,
        A: Clone,
    {
        let cached: Option<T> = self.cache_find(&T::query(query_args.clone()));

        if let Some(c) = cached {
            println!("GOT CACHED {:?}", cached);
            return c;
        }

        let PgQuery { query, args } = T::query(query_args.clone());

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

        trans.finish().expect("Tranny finished");

        self.cache_save(&T::query(query_args.clone()), &results);

        results
    }
}
