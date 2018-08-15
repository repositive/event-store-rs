#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use serde::Deserialize;
use std::fmt::Debug;
use std::marker::PhantomData;

// --- Event store crate ---
pub trait Event {}
pub trait Events {}
pub trait StoreQuery {}
pub struct PgQuery(pub String);
pub trait Aggregator<E: Events, A, Q: StoreQuery>: Copy + Clone + Debug + Default {
    fn apply_event(acc: Self, event: &E) -> Self;

    fn query() -> Q;
}

impl StoreQuery for PgQuery {}

pub trait Store<E: Events> {
    fn new() -> Self;

    fn aggregate<T, A, Q: StoreQuery>(&self, query: A) -> T
    where
        E: Events,
        T: Aggregator<E, A, Q>;
}

pub struct PgStore<E: Events> {
    phantom: PhantomData<E>,
}

impl<'a, E> Store<E> for PgStore<E>
where
    E: Events + Deserialize<'a>,
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn aggregate<T, A, Q: StoreQuery>(&self, _query: A) -> T
    where
        T: Aggregator<E, A, Q>,
    {
        let inc: E = serde_json::from_str(
            r#"{
            "type": "some_namespace.Inc",
            "by": 1
        }"#,
        ).unwrap();
        let dec: E = serde_json::from_str(
            r#"{
            "type": "some_namespace.Dec",
            "by": 1
        }"#,
        ).unwrap();

        let events = vec![inc, dec];

        let result = events
            .iter()
            .fold(T::default(), |acc, event| T::apply_event(acc, event));

        result
    }
}
