//! Store adapter backed by Postgres

use adapters::pg::PgQuery;
use adapters::{CacheResult, StoreAdapter};
use chrono::{DateTime, Utc};
use fallible_iterator::FallibleIterator;
use futures::future::{err as FutErr, lazy as NewFuture, ok as FutOk, Future, FutureResult};
use postgres::error::DUPLICATE_COLUMN;
use postgres::types::ToSql;
use r2d2;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use serde_json::{from_value, to_value, Value as JsonValue};
use utils::BoxedFuture;
use uuid::Uuid;
use Aggregator;
use Event;
use EventContext;
use EventData;
use Events;

/// Postgres store adapter
#[derive(Clone)]
pub struct PgStoreAdapter {
    pool: r2d2::Pool<PostgresConnectionManager>,
}

impl<'a> PgStoreAdapter {
    /// Create a new PgStore from a Postgres DB connection
    pub fn new(conn: PostgresConnectionManager) -> Self {
        Self {
            pool: r2d2::Pool::new(conn).unwrap(),
        }
    }

    //    fn generate_query<E, T, A>(query_string: &PgQuery<'a>, since: Utc) -> (T, String)
    //    where
    //        E: Events,
    //        T: Aggregator<E, A, PgQuery<'a>> + Default,
    //        A: Clone,
    //    {
    //        let (initial_state, query_string) = if let Some((existing, time)) = since {
    //            (existing, format!(
    //                "SELECT * FROM ({}) AS events WHERE events.context->>'time' >= '{}' ORDER BY events.context->>'time' ASC",
    //                query_string.query, time
    //            ))
    //        } else {
    //            (T::default(), String::from(query_string.query))
    //        };
    //
    //        (initial_state, query_string)
    //    }

    fn generate_string_query(initial_query: &PgQuery<'a>, since: Utc) -> String {
        String::from(format!(
            "SELECT * FROM ({}) AS events WHERE events.context->>'time' >= '{}' ORDER BY events.context->>'time' ASC",
            initial_query.query, since,
        ))
    }
}

impl<'a> StoreAdapter<PgQuery<'a>> for PgStoreAdapter {
    fn read<'b, E>(&self, query: PgQuery<'b>, since: Utc) -> BoxedFuture<'b, Vec<E>, String>
    where
        E: Events + Send + 'b,
    {
        let conn = self.pool.get();
        Box::from(NewFuture(|| {
            let pool = conn.expect("Could not connect to the pool (aggregate)");

            let query_string = Self::generate_string_query(&query, since);
            let trans = pool
                .transaction()
                .expect("Unable to initialise transaction");
            let stmt = trans
                .prepare(&query_string)
                .expect("Unable to prepare transaction");
            let mut params: Vec<&ToSql> = Vec::new();

            for (i, _arg) in query.args.iter().enumerate() {
                params.push(&*query.args[i]);
            }

            let results = stmt
                .lazy_query(&trans, &params, 1000)
                .unwrap()
                .map(|row| {
                    let id: Uuid = row.get("id");
                    let data_json: JsonValue = row.get("data");
                    let context_json: JsonValue = row.get("context");

                    let thing = json!({
                            "id": id,
                            "data": data_json,
                            "context": context_json,
                        });

                    let evt: E = from_value(thing).expect("Could not decode row");

                    evt
                }).collect()
                .expect("ain't no collec");

            trans.finish().expect("Could not finish transaction");

            FutOk(results)
        }))
    }

    fn save<'b, ED: EventData + Sync + Send + 'b>(
        &self,
        event: &'b Event<ED>,
    ) -> BoxedFuture<'b, (), String> {
        let conn = self.pool.clone();
        Box::from(NewFuture(move || {
            let res = conn
                .get()
                .expect("Could not connect to the pool (save)")
                .execute(
                    r#"INSERT INTO events (id, data, context)
                    VALUES ($1, $2, $3)"#,
                    &[
                        &event.id,
                        &to_value(&event.data).expect("Unable to convert event data to value"),
                        &to_value(&event.context).expect("Cannot convert event context"),
                    ],
                ).map(|_| ())
                .map_err(|err| match err.code() {
                    Some(e) if e == &DUPLICATE_COLUMN => "DUPLICATE_COLUMN".into(),
                    _ => "UNEXPECTED".into(),
                });
            match res {
                Ok(_) => FutOk(()),
                Err(s) => FutErr(s),
            }
        }))
    }

    fn last_event<ED: EventData + Send + 'static>(&self) -> BoxedFuture<Option<Event<ED>>, String> {
        let initial_future = FutOk(());
        initial_future;

        let conn = self.pool.clone();
        Box::from(
            FutOk(()).and_then(
                |_| {
                    let rows = conn
                .get()
                .expect("Could not connect to the pool (last_event)")
                .query(
                    r#"SELECT * from events where data->>'event_namespace' = $1 and data->>'event_type' = $2 order by data->>'time' desc limit 1
                    "#,
                    &[
                        &ED::event_namespace(),
                        &ED::event_type()
                    ],
                    ).expect("Responsen't");
                    if rows.len() == 1 {
                        let row = rows.get(0);
                        let id: Uuid = row.get("id");
                        let data_json: JsonValue = row.get("data");
                        let context_json: JsonValue = row.get("context");

                        let data: ED = from_value(data_json).unwrap();
                        let context: EventContext = from_value(context_json).unwrap();
                        FutOk(Some(Event { id, data, context }))
                    } else {
                        FutOk(None)
                    }
                },
        ))
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
