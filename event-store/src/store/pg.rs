use chrono::{DateTime, Utc};
use crate::Event;
use crate::TestEvent;
use crate::TestEvents;
use fallible_iterator::FallibleIterator;
use postgres::error::{DUPLICATE_COLUMN, UNIQUE_VIOLATION};
use postgres::types::ToSql;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use serde_json::{from_value, to_value, Value as JsonValue};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug)]
pub struct PgQuery {
    pub query: String,
    pub args: Vec<Box<ToSql>>,
}

impl PgQuery {
    pub fn new(query: &str, args: Vec<Box<ToSql>>) -> Self {
        Self {
            query: query.into(),
            args,
        }
    }

    pub fn unique_id(&self) -> String {
        let hash = Sha256::digest(format!("{:?}:[{}]", self.args, self.query).as_bytes());

        hash.iter().fold(String::new(), |mut acc, hex| {
            acc.push_str(&format!("{:X}", hex));
            acc
        })
    }
}

#[derive(Debug, Clone)]
pub struct PgStoreAdapter {
    pool: Pool<PostgresConnectionManager>,
}

impl PgStoreAdapter {
    pub fn new(pool: Pool<PostgresConnectionManager>) -> Self {
        Self { pool }
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

    pub fn read(
        &self,
        query: PgQuery,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<TestEvents>, String> {
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

                let evt: TestEvents = from_value(thing).expect("Could not decode row");

                evt
            })
            .collect()
            .expect("ain't no collec");

        trans.finish().expect("Could not finish transaction");

        Ok(results)
    }

    pub fn save(&self, event: &Event<TestEvent>) -> Result<(), String> {
        trace!("Persist event to store {:?}", event);

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
            .map_err(|err| match err.code() {
                Some(e) if e == &DUPLICATE_COLUMN || e == &UNIQUE_VIOLATION => {
                    debug!("Event {} already exists", event.id);

                    "DUPLICATE_COLUMN".into()
                }
                _ => "UNEXPECTED".into(),
            })
    }

    pub fn last_event(&self) -> Result<Option<Event<TestEvent>>, String> {
        Ok(None)
    }
}
