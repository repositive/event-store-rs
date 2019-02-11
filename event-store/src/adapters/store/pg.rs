use crate::event::Event;
use crate::event_context::EventContext;
use crate::store_query::StoreQuery;
use chrono::prelude::*;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use fallible_iterator::FallibleIterator;
use log::{debug, trace};
use postgres::error::UNIQUE_VIOLATION;
use r2d2::Pool;
use r2d2_postgres::postgres::types::ToSql;
use r2d2_postgres::PostgresConnectionManager;
use serde_json::{from_value, json, to_value, Value as JsonValue};
use sha2::{Digest, Sha256};
use std::io;
use uuid::Uuid;

const INIT_QUERIES: &'static str = r#"
-- Create UUID extension just in case
create extension if not exists "uuid-ossp";

-- Create events table if it doesn't already exist
create table if not exists events(
    id uuid default uuid_generate_v4() primary key,
    data jsonb not null,
    context jsonb default '{}'
);

-- Add sequence number column
alter table events add column if not exists sequence_number bigserial;

-- Add index on sequence number and time to speed up ordering
create index if not exists counter_time on events ((context->>'time') asc);

-- Create index to speed up queries by type
create index if not exists event_type_legacy on events ((data->>'type') nulls last);
create index if not exists event_namespace_and_type on events ((context->>'event_namespace') nulls last, (context->>'event_type') nulls last);

-- Create last event log table
create table if not exists last_handled_event_log(
    domain varchar(64) not null,
    event_namespace varchar(64) not null,
    event_type varchar(64) not null,
    event_id uuid not null,
    time timestamp with time zone not null,
    sequence_number bigint not null,
    primary key(domain, event_namespace, event_type)
);
"#;

/// Representation of a Postgres query and args
#[derive(Debug)]
pub struct PgQuery {
    /// Query string with placeholders
    pub query: String,

    /// Arguments to use for the query
    pub args: Vec<Box<ToSql + Send + Sync>>,
}

impl PgQuery {
    /// Create a new query from a query string and arguments
    pub fn new(query: &str, args: Vec<Box<ToSql + Send + Sync>>) -> Self {
        Self {
            query: query.into(),
            args,
        }
    }
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

fn generate_query(initial_query: &PgQuery, since: Option<DateTime<Utc>>) -> String {
    if let Some(timestamp) = since {
        String::from(format!(
            "select * from ({}) as events where (events.context->>'time')::timestamp with time zone >= '{}' order by (events.context->>'time')::timestamp with time zone asc",
            initial_query.query, timestamp,
        ))
    } else {
        String::from(format!(
            "select * from ({}) as events order by (events.context->>'time')::timestamp with time zone asc",
            initial_query.query
        ))
    }
}

/// Save result
pub enum SaveStatus {
    /// The save was successful
    Ok,

    /// A duplicate item already exists in the backing store
    Duplicate,
}

/// The result of a save operation
///
/// If the save did not error but a duplicate was encountered, this should be equal to
/// `Ok(SaveStatus::Duplicate)`
pub type SaveResult = Result<SaveStatus, io::Error>;

/// Postgres-backed store adapter
#[derive(Clone)]
pub struct PgStoreAdapter {
    conn: Pool<PostgresConnectionManager>,
}

impl PgStoreAdapter {
    /// Create a new Postgres store
    ///
    /// This will attempt to create the events table and indexes if they do not already exist
    pub async fn new(conn: Pool<PostgresConnectionManager>) -> Result<Self, io::Error> {
        conn.get()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
            .batch_execute(INIT_QUERIES)?;

        Ok(Self { conn })
    }

    /// Save an event into PG
    pub fn save<'a, ED>(&'a self, event: &'a Event<ED>) -> SaveResult
    where
        ED: EventData,
    {
        debug!(
            "Insert event {}.{}",
            ED::event_namespace(),
            ED::event_type()
        );

        self.conn
            .get()
            .unwrap()
            .execute(
                "insert into events (id, data, context) values ($1, $2, $3)",
                &[
                    &event.id,
                    &to_value(&event.data).expect("Unable to convert event data to value"),
                    &to_value(&event.context).expect("Cannot convert event context"),
                ],
            )
            .map(|_| Ok(SaveStatus::Ok))
            .unwrap_or_else(|err| {
                let is_duplicate_error = err.code().unwrap() == &UNIQUE_VIOLATION;

                if is_duplicate_error {
                    Ok(SaveStatus::Duplicate)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Could not save event: {}", err),
                    ))
                }
            })
    }

    /// Read a list of events
    pub async fn read<'a, E>(
        &'a self,
        query: &'a PgQuery,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<E>, io::Error>
    where
        E: Events,
    {
        let query_string = generate_query(&query, since);

        debug!("Read query {}", query_string);

        let conn = self.conn.get().unwrap();

        let trans = conn
            .transaction()
            .expect("Unable to initialise transaction");

        let stmt = trans
            .prepare(&query_string)
            .expect("Unable to prepare read statement");

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
            .expect("Failed to collect results");

        trans.finish().expect("Could not finish transaction");

        Ok(results)
    }

    /// Find the most recent event of a given type
    pub fn last_event<ED>(&self) -> Result<Option<Event<ED>>, io::Error>
    where
        ED: EventData,
    {
        let rows = self
            .conn
            .get()
            .unwrap()
            .query(
                r#"select * from events
                    where data->>'event_namespace' = $1
                    and data->>'event_type' = $2
                    order by (context->>'time')::timestamp with time zone desc
                    limit 1"#,
                &[&ED::event_namespace(), &ED::event_type()],
            )
            .expect("Unable to query database (last_event)");

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

    /// Fetch events of a given type starting from a timestamp going forward
    pub async fn read_events_since<'a>(
        &'a self,
        event_namespace: &'a str,
        event_type: &'a str,
        since: DateTime<Utc>,
    ) -> Result<Vec<JsonValue>, io::Error> {
        let query_string = r#"select * from events
            where data->>'event_namespace' = $1
            and data->>'event_type' = $2
            and context->>'time' >= $3
            order by (context->>'time')::timestamp with time zone asc"#;

        let conn = self.conn.get().unwrap();

        let trans = conn
            .transaction()
            .expect("Unable to initialise transaction");

        let stmt = trans
            .prepare(&query_string)
            .expect("Unable to prepare read statement");

        trace!(
            "Read events of type {}.{} since {}",
            event_namespace,
            event_type,
            since.to_rfc3339()
        );

        let results = stmt
            .lazy_query(
                &trans,
                &[&event_namespace, &event_type, &since.to_rfc3339()],
                1000,
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
            .map(|row| {
                let id: Uuid = row.get("id");
                let data_json: JsonValue = row.get("data");
                let context_json: JsonValue = row.get("context");

                json!({
                    "id": id,
                    "data": data_json,
                    "context": context_json,
                })
            })
            .collect()
            .expect("Failed to collect results");

        trans.finish().expect("Could not finish transaction");

        Ok(results)
    }
}
