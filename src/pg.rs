//! Postgres-backed event store

use super::{Aggregator, EventContext, Events, Store};
use adapters::PgCacheAdapter;
use adapters::PgQuery;
use adapters::PgStoreAdapter;
use adapters::StubEmitterAdapter;
use adapters::{CacheAdapter, EmitterAdapter, StoreAdapter};
use chrono::prelude::*;
use postgres::Connection;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_json::to_value;
use std::marker::PhantomData;
use uuid::Uuid;

// /// Representation of a Postgres query and args
// pub struct PgQuery<'a> {
//     /// Query string with placeholders
//     query: &'a str,

//     /// Arguments to use for the query
//     args: Vec<Box<ToSql>>,
// }

// impl<'a> StoreQuery for PgQuery<'a> {}

// impl<'a> PgQuery<'a> {
//     /// Create a new query from a query string and arguments
//     pub fn new(query: &'a str, args: Vec<Box<ToSql>>) -> Self {
//         Self { query, args }
//     }
// }

/// Postgres-backed event store
pub struct PgStore<E: Events> {
    phantom: PhantomData<E>,
    conn: Connection,
    cache: PgCacheAdapter,
    store: PgStoreAdapter,
    emitter: StubEmitterAdapter,
}

impl<'a, E> PgStore<E>
where
    E: Events + Deserialize<'a>,
{
    /// Create a new PgStore from a Postgres DB connection
    pub fn new(
        conn: Connection,
        store: PgStoreAdapter,
        cache: PgCacheAdapter,
        emitter: StubEmitterAdapter,
    ) -> Self {
        Self {
            phantom: PhantomData,
            conn,
            cache,
            store,
            emitter,
        }
    }

    // fn cache_save<T>(&self, q: &PgQuery, result: &T)
    // where
    //     T: Serialize,
    // {
    //     let args_hash = Sha256::digest(format!("{:?}:[{}]", q.args, q.query).as_bytes());

    //     self.conn
    //         .execute(
    //             r#"INSERT INTO aggregate_cache (id, data, time)
    //             VALUES ($1, $2, NOW())
    //             ON CONFLICT (id)
    //             DO UPDATE SET data = EXCLUDED.data, time = now() RETURNING data"#,
    //             &[&args_hash.as_slice(), &to_value(result).expect("To value")],
    //         ).expect("Cache");
    // }

    // fn cache_find<T>(&self, q: &PgQuery) -> Option<(T, NaiveDateTime)>
    // where
    //     T: DeserializeOwned + Default,
    // {
    //     let args_hash = Sha256::digest(format!("{:?}:[{}]", q.args, q.query).as_bytes());

    //     let rows = self
    //         .conn
    //         .query(
    //             "SELECT data, time FROM aggregate_cache WHERE id = $1 LIMIT 1",
    //             &[&args_hash.as_slice()],
    //         ).expect("Ret");

    //     // `rows.get()` panics if index is out of bounds, hence this check. Should be an Option.
    //     if rows.len() != 1 {
    //         None
    //     } else {
    //         let row = rows.get(0);

    //         let time: NaiveDateTime = row.get(1);

    //         Some((
    //             from_value(row.get(0)).map(|decoded: T| decoded).unwrap(),
    //             time,
    //         ))
    //     }
    // }
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
        let q = T::query(query_args.clone());

        let result: T = self
            .store
            .aggregate(&q, self.cache.get(&q))
            .expect("Aggregate");

        result
    }

    fn save<C>(&self, item: E, subject: Option<C>) -> Result<(), String>
    where
        C: Serialize,
    {
        let time: DateTime<Utc> = Utc::now();
        let context = EventContext {
            action: None,
            subject: subject.map(|s| to_value(s).expect("Could not serialize subject")),
            time,
        };
        let id = Uuid::new_v4();

        self.conn
            .execute(
                r#"INSERT INTO events (id, data, context)
                VALUES ($1, $2, $3)"#,
                &[
                    &id,
                    &to_value(&item).expect("Item to value"),
                    &to_value(context).expect("Context to value"),
                ],
            ).expect("Save");

        self.emitter.emit(&item);

        Ok(())
    }
}
