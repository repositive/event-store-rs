extern crate fallible_iterator;
extern crate postgres;
extern crate serde;
extern crate serde_json;

use fallible_iterator::FallibleIterator;
use postgres::types::ToSql;
use postgres::Connection;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{from_value, Value as JsonValue};
use std::fmt::Debug;
use std::marker::PhantomData;

pub trait Event {}
pub trait Events {}
pub trait StoreQuery {}
pub trait Aggregator<E: Events, A, Q: StoreQuery>: Copy + Clone + Debug + Default {
    fn apply_event(acc: Self, event: &E) -> Self;

    fn query(field: A) -> Q;
}

pub struct PgQuery<'a> {
    query: &'a str,
    args: Vec<Box<ToSql>>,
}

impl<'a> StoreQuery for PgQuery<'a> {}

impl<'a> PgQuery<'a> {
    pub fn new(query: &'a str, args: Vec<Box<ToSql>>) -> Self {
        Self { query, args }
    }
}

pub trait Store<E: Events, Q: StoreQuery> {
    fn aggregate<T, A>(&self, query: A) -> T
    where
        E: Events,
        T: Aggregator<E, A, Q>;
}

pub struct PgStore<E: Events> {
    phantom: PhantomData<E>,
    conn: Connection,
}

impl<'a, E> PgStore<E>
where
    E: Events + Deserialize<'a>,
{
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
