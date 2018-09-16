//! Store adapter backed by Postgres

use adapters::StoreAdapter;
use chrono::Utc;
use futures::future::{ok as FutOk, result as FutResult, Future};
use futures::stream::{empty, Stream};
// use postgres::error::DUPLICATE_COLUMN;
use bb8::Pool;
use bb8_postgres::tokio_postgres::rows::Row as PgRow;
use bb8_postgres::PostgresConnectionManager;
use futures_state_stream::StateStream;
use serde_json::{from_value, to_value, Value as JsonValue};
use utils::{BoxedFuture, BoxedStream};

use uuid::Uuid;

use Event;
// use EventContext;
use super::StoreQuery;
use EventContext;
use EventData;
use Events;

/// Postgres store adapter
pub struct PgStoreAdapter {
    pool: Pool<PostgresConnectionManager>,
}

impl<'a> PgStoreAdapter {
    /// Create a new PgStore from a Postgres DB connection
    pub fn new(pool: Pool<PostgresConnectionManager>) -> Self {
        Self { pool }
    }
}

impl<'s> StoreAdapter<&'s str> for PgStoreAdapter {
    fn read<'a, E: Events + 'a, A, Q: StoreQuery<'a, &'s str, A>, H: Fn(E) -> () + 'a>(
        &self,
        query_args: A,
        since: Utc,
        handler: H,
    ) -> BoxedFuture<'a, (), String> {
        // Somehow this is meant to turn args: A -> A stream of events.
        //  Get a stream from the db
        //  map stream of records to stream of events
        //  return that mapped stream
        //  also handle errors

        let task = self.pool.run(|connection| {
            connection
                .prepare("SELECT 1")
                .and_then(|(select, connection)| {
                    let stream = connection.query(&select, &[]).map(|row| {
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
                    });

                    stream.for_each(handler)
                }).map(|connection| ((), connection))
        });

        Box::new(task.map_err(|_| String::from("Internal tokio error")))
    }

    //fn save<ED: EventData>(&self, event: &Event<ED>) -> Arc<Future<Item = (), Error = String>> {
    fn save<'a, ED: EventData + 'a>(&self, event: Event<ED>) -> BoxedFuture<'a, (), String> {
        // Logic for this fn
        let result = self
            .pool
            .run(move |connection| {
                connection
                    .prepare("INSERT INTO events (id, data, context) VALUES ($1, $2, $3)")
                    .and_then(move |(insert, connection)| {
                        connection
                            .query(
                                &insert,
                                &[
                                    &event.id,
                                    &to_value(&event.data).expect("Item to value"),
                                    &to_value(&event.context).expect("Context to value"),
                                ],
                            ).for_each(|_| ())
                    }).map(|connection| ((), connection))
            }).map(|_| ())
            .map_err(|_| String::from("Failed to insert event"));
        Box::new(result)
    }

    fn last_event<'a, ED: EventData + 'a>(&self) -> BoxedFuture<'a, Option<Event<ED>>, String> {
        // Utility functions for this fn
        fn extract_one_event<ED: EventData>(row: &PgRow) -> Event<ED> {
            let id: Uuid = row.get("id");
            let data_json = row.get("data");
            let context_json = row.get("context");

            let data: ED = from_value(data_json).unwrap();
            let context: EventContext = from_value(context_json).unwrap();

            Event { id, data, context }
        }

        let result = self.pool
            // XXX: This fn is defined at https://github.com/khuey/bb8/blob/master/src/lib.rs#L706
            // approach with caution!
            .run(|connection| {
                connection
                    .prepare("SELECT * FROM events WHERE data->>'event_namespace' = $1 AND data->>'event_type' = $2 ORDER BY data->>'time' desc limit 1")
                    .and_then(|(select, conn)| {
                        let result = conn
                            .query(
                                &select,
                                &[
                                    &ED::event_namespace(),
                                    &ED::event_type(),
                                ],
                            )
                            .collect().map(|(vec, connection)| (vec.first().map(extract_one_event), connection));
                        result
                    })
            // TODO: properly handle errors
            })
            .map_err(|_| String::from("Error"));
        Box::new(result)
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
