//! Store adapter backed by Postgres

use super::StoreQuery;
use adapters::StoreAdapter;
use chrono::{DateTime, NaiveDateTime, Utc};
use fallible_iterator::FallibleIterator;
use postgres::error::{DUPLICATE_COLUMN, UNIQUE_VIOLATION};
use postgres::types::ToSql;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use serde_json::{from_value, to_value, Value as JsonValue};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use Event;
use EventContext;
use EventData;
use Events;

/// Representation of a Postgres query and args
#[derive(Debug)]
pub struct PgQuery {
    /// Query string with placeholders
    pub query: String,

    /// Arguments to use for the query
    pub args: Vec<Box<ToSql>>,
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

impl PgQuery {
    /// Create a new query from a query string and arguments
    pub fn new(query: &str, args: Vec<Box<ToSql>>) -> Self {
        Self {
            query: query.into(),
            args,
        }
    }
}

/// Postgres store adapter
#[derive(Clone)]
pub struct PgStoreAdapter {
    pool: Pool<PostgresConnectionManager>,
}

impl PgStoreAdapter {
    /// Create a new PgStore from a Postgres DB connection
    pub fn new(conn: Pool<PostgresConnectionManager>) -> Self {
        Self { pool: conn }
    }

    fn generate_query(initial_query: &PgQuery, since: Option<DateTime<Utc>>) -> String {
        if let Some(timestamp) = since {
            String::from(format!(
            "SELECT * FROM ({}) AS events WHERE events.context->>'time' >= '{}' ORDER BY events.context->>'time' ASC",
            initial_query.query, timestamp,
        ))
        } else {
            String::from(format!(
                "SELECT * FROM ({}) AS events ORDER BY events.context->>'time' ASC",
                initial_query.query
            ))
        }
    }
}

impl StoreAdapter<PgQuery> for PgStoreAdapter {
    fn read<E>(&self, query: PgQuery, since: Option<DateTime<Utc>>) -> Result<Vec<E>, String>
    where
        E: Events,
    {
        let conn = self.pool.clone();

        let pool = conn
            .get()
            .expect("Could not connect to the pool (aggregate)");

        let query_string = Self::generate_query(&query, since);
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
            })
            .collect()
            .expect("ain't no collec");

        trans.finish().expect("Could not finish transaction");

        Ok(results)
    }

    fn save<ED>(&self, event: &Event<ED>) -> Result<(), String>
    where
        ED: EventData,
    {
        self.pool
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
            )
            .map(|_| ())
            .map_err(|err| {
                error!("Store save error: code {:?}", err);

                match err.code() {
                    Some(e) if e == &DUPLICATE_COLUMN || e == &UNIQUE_VIOLATION => {
                        "DUPLICATE_COLUMN".into()
                    }
                    _ => "UNEXPECTED".into(),
                }
            })
    }

    fn last_event<ED>(&self) -> Result<Option<Event<ED>>, String>
    where
        ED: EventData,
    {
        let rows = self.pool
                .get()
                .expect("Could not connect to the pool (last_event)")
                .query(
                    r#"SELECT * from events where data->>'event_namespace' = $1 and data->>'event_type' = $2 order by data->>'time' desc limit 1
                    "#,
                    &[
                        &ED::event_namespace(),
                        &ED::event_type()
                    ],
                ).expect("Unable to query database (last_event)");

        if rows.len() == 1 {
            let row = rows.get(0);
            let id: Uuid = row.get("id");
            let data_json: JsonValue = row.get("data");
            let context_json: JsonValue = row.get("context");

            let data: ED = from_value(data_json).unwrap();
            let context: EventContext = from_value(context_json).unwrap();
            Ok(Some(Event { id, data, context }))
        } else {
            Ok(None)
        }
    }

    fn read_since<ED>(&self, since: Option<DateTime<Utc>>) -> Result<Vec<Event<ED>>, String>
    where
        ED: EventData,
    {
        // Last found event time, or start from the beginning of time if none
        let from_time: DateTime<Utc> = since.unwrap_or(DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(0, 0),
            Utc,
        ));

        let conn = self.pool.clone();

        let pool = conn
            .get()
            .expect("Could not connect to the pool (aggregate)");

        let trans = pool
            .transaction()
            .expect("Unable to initialise transaction");

        let stmt = trans
            .prepare(
                r#"SELECT * FROM events
                WHERE events.context->>'time' >= $1
                AND data->>'event_namespace' = $2
                AND data->>'event_type' = $3
                ORDER BY events.context->>'time' ASC
            "#,
            )
            .expect("Unable to prepare transaction");

        let results = stmt
            .lazy_query(
                &trans,
                &[&ED::event_namespace(), &ED::event_type(), &from_time],
                1000,
            )
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

                let evt: Event<ED> = from_value(thing).expect("Could not decode row");

                evt
            })
            .collect()
            .expect("ain't no collec");

        trans.finish().expect("Could not finish transaction");

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aggregator::Aggregator;
    use chrono::prelude::*;
    use testhelpers::*;

    #[test]
    fn it_generates_a_query_when_there_is_no_cache() {
        let q = TestCounterEntity::query("something".into());
        let since = None;

        let query_string: String = PgStoreAdapter::generate_query(&q, since);

        let base_query = "select * from events where data->>'ident' = $1";
        let expected_query = format!(
            "SELECT * FROM ({}) AS events ORDER BY events.context->>'time' ASC",
            base_query
        );

        assert_eq!(query_string, expected_query);
    }

    #[test]
    fn it_generates_a_different_query_when_there_is_a_cache() {
        let q = TestCounterEntity::query("something".into());

        let since = Some(Utc.ymd(2018, 8, 27).and_hms(12, 43, 52));

        let query_string = PgStoreAdapter::generate_query(&q, since);

        let base_query = "select * from events where data->>'ident' = $1";
        let generated_query = format!(
            "SELECT * FROM ({}) AS events WHERE events.context->>'time' >= '{}' ORDER BY events.context->>'time' ASC",
            base_query, "2018-08-27 12:43:52 UTC");

        assert_eq!(query_string, generated_query);
    }
}
