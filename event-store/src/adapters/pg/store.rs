//! Store adapter backed by Postgres

use super::Connection;
use adapters::pg::PgQuery;
use adapters::{CacheResult, StoreAdapter};
use fallible_iterator::FallibleIterator;
use futures::future::ok as FutOk;
use postgres::types::ToSql;
use serde_json::{from_value, to_value, Value as JsonValue};
use std::marker::PhantomData;
use utils::BoxedFuture;
use uuid::Uuid;
use Aggregator;
use Event;
use EventContext;
use EventData;
use Events;

/// Postgres store adapter
pub struct PgStoreAdapter<E> {
    phantom: PhantomData<E>,
    conn: Connection,
}

impl<'a, E> PgStoreAdapter<E>
where
    E: Events,
{
    /// Create a new PgStore from a Postgres DB connection
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            phantom: PhantomData,
        }
    }

    fn generate_query<T, A>(
        query_string: &PgQuery<'a>,
        since: Option<CacheResult<T>>,
    ) -> (T, String)
    where
        T: Aggregator<E, A, PgQuery<'a>> + Default,
        A: Clone,
    {
        let (initial_state, query_string) = if let Some((existing, time)) = since {
            (existing, format!(
                "SELECT * FROM ({}) AS events WHERE events.context->>'time' >= '{}' ORDER BY events.context->>'time' ASC",
                query_string.query, time
            ))
        } else {
            (T::default(), String::from(query_string.query))
        };

        (initial_state, query_string)
    }
}

impl<'a, E> StoreAdapter<E, PgQuery<'a>> for PgStoreAdapter<E>
where
    E: Events,
{
    fn aggregate<T, A>(&self, query_args: A, since: Option<CacheResult<T>>) -> Result<T, String>
    where
        T: Aggregator<E, A, PgQuery<'a>> + Default,
        A: Clone,
    {
        let q = T::query(query_args);
        let (initial_state, query_string) = Self::generate_query(&q, since);

        let conn = self.conn.get().expect("Could not get PG connection");

        let trans = conn.transaction().expect("Tranny");
        let stmt = trans.prepare(&query_string).expect("Prep");

        let mut params: Vec<&ToSql> = Vec::new();

        for (i, _arg) in q.args.iter().enumerate() {
            params.push(&*q.args[i]);
        }

        let results = stmt
            .lazy_query(&trans, &params, 1000)
            .expect("Query")
            .map(|row| {
                let id: Uuid = row.get("id");
                let data_json: JsonValue = row.get("data");
                let context_json: JsonValue = row.get("context");

                let data: E = from_value(data_json).unwrap();
                let context: EventContext = from_value(context_json).unwrap();

                Event { id, data, context }
            })
            .fold(initial_state, |acc, event| T::apply_event(acc, &event))
            .expect("Fold");

        trans.finish().expect("Tranny finished");

        Ok(results)
    }

    fn save(&self, event: &Event<E>) -> Result<(), String> {
        self.conn
            .get()
            .expect("Could not get PG connection")
            .execute(
                r#"INSERT INTO events (id, data, context)
                VALUES ($1, $2, $3)"#,
                &[
                    &event.id,
                    &to_value(&event.data()).expect("Item to value"),
                    &to_value(&event.context()).expect("Context to value"),
                ],
            )
            .expect("Save");

        Ok(())
    }

    fn last_event<ED: EventData + Send + 'static>(&self) -> BoxedFuture<Option<Event<ED>>, String> {
        self.conn
            .execute(
                r#"SELECT * from events where data->>'event_namespace' = $1 and data->>'event_type' = $2
                "#,
                &[
                    &ED::event_namespace(),
                    &ED::event_type()
                ],
                ).expect("Response");
        Box::new(FutOk(None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;
    use testhelpers::*;

    #[test]
    fn it_generates_a_query_when_there_is_no_cache() {
        let q = TestCounterEntity::query("something".into());
        let since = None;

        let (state, query_string): (TestCounterEntity, String) =
            PgStoreAdapter::generate_query(&q, since);

        let expected_query = "select * from events where data->>'ident' = $1";

        assert_eq!(state, TestCounterEntity::default());

        assert_eq!(query_string, expected_query);
    }

    #[test]
    fn it_generates_a_different_query_when_there_is_a_cache() {
        let q = TestCounterEntity::query("something".into());
        let since: Option<CacheResult<TestCounterEntity>> = Some((
            TestCounterEntity::default(),
            Utc.ymd(2018, 8, 27).and_hms(12, 43, 52),
        ));

        let (state, query_string) = PgStoreAdapter::generate_query(&q, since);

        let base_query = "select * from events where data->>'ident' = $1";
        let generated_query = format!(
            "SELECT * FROM ({}) AS events WHERE events.context->>'time' >= '{}' ORDER BY events.context->>'time' ASC",
            base_query, "2018-08-27 12:43:52 UTC");
        assert_eq!(state, TestCounterEntity::default()); // What does this end up being?

        assert_eq!(query_string, generated_query);
    }
}
