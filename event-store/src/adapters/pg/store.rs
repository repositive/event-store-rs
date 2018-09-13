//! Store adapter backed by Postgres

use super::Connection;
use adapters::pg::PgQuery;
use adapters::{CacheResult, StoreAdapter};
use chrono::Utc;
use futures::future::{ok as FutOk, Future};
use futures::stream::empty;
// use postgres::error::DUPLICATE_COLUMN;
use serde_json::{from_value, to_value, Value};
use std::sync::{Arc, Mutex};
use tokio_postgres::types::ToSql;
use utils::{BoxedFuture, BoxedStream};
use uuid::Uuid;
use Aggregator;

use Event;
// use EventContext;
use EventData;
use Events;

/// Postgres store adapter
#[derive(Clone)]
pub struct PgStoreAdapter {
    conn: Arc<Connection>,
}

impl<'a> PgStoreAdapter {
    /// Create a new PgStore from a Postgres DB connection
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }
}

impl StoreAdapter for PgStoreAdapter {
    fn read<'a, E: Events + Send + 'a, A: Clone>(
        &self,
        args: A,
        since: Utc,
    ) -> BoxedStream<'a, E, String> {
        Box::new(empty())
    }

    fn save<ED: EventData>(&self, event: &Event<ED>) -> BoxedFuture<(), String> {
        let connection_clone = &self.conn.clone();
        let connection_lock = connection_clone.lock().unwrap();
        let task: BoxedFuture<(), String> = Box::new(
            connection_lock
                .prepare("INSERT INTO events (id, data, context) VALUES ($1, $2, $3)")
                .and_then(|(insert, connection)| {
                    // connection
                    //     .query(
                    //         &insert,
                    //         &[
                    //             &event.id.clone(),
                    //             &to_value(&event.data.clone()).expect("Item to value"),
                    //             &to_value(&event.context.clone()).expect("Context to value"),
                    //         ],
                    //     ).into()
                    FutOk(())
                }).map_err(|_| String::from("Boom")),
        );
        task
        // .and_then(|(insert, connection)| {
        //     connection.query(
        //         &insert,
        //     )
        // }).map_err(|_| String::from("Boom!"))
        // .map(|_| ());
        // Box::new(FutOk(()))
        //self.conn
        //    .get()
        //    .expect("Could not get PG connection")
        //    .execute(
        //        r#"INSERT INTO events (id, data, context)
        //        VALUES ($1, $2, $3)"#,
        //        &[
        //            &event.id,
        //            &to_value(&event.data).expect("Item to value"),
        //            &to_value(&event.context).expect("Context to value"),
        //        ],
        //    ).map(|_| ())
        //    .map_err(|err| match err.code() {
        //        Some(e) if e == &DUPLICATE_COLUMN => "DUPLICATE_COLUMN".into(),
        //        _ => "UNEXPECTED".into(),
        //    })
    }

    fn last_event<ED: EventData + 'static>(&self) -> BoxedFuture<Option<Event<ED>>, String> {
        Box::new(FutOk(None))
        // let rows = self.conn
        //     .get()
        //     .expect("Could not get PG connection")
        //     .query(
        //         r#"SELECT * from events where data->>'event_namespace' = $1 and data->>'event_type' = $2 order by data->>'time' desc limit 1
        //         "#,
        //         &[
        //             &ED::event_namespace(),
        //             &ED::event_type()
        //         ],
        //         ).expect("Response");
        // if rows.len() == 1 {
        //     let row = rows.get(0);
        //     let id: Uuid = row.get("id");
        //     let data_json: JsonValue = row.get("data");
        //     let context_json: JsonValue = row.get("context");

        //     let data: ED = from_value(data_json).unwrap();
        //     let context: EventContext = from_value(context_json).unwrap();

        //     Box::new(FutOk(Some(Event { id, data, context })))
        // } else {
        //     Box::new(FutOk(None))
        // }
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
