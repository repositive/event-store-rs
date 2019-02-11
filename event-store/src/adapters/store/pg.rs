use super::LastHandledEvent;
use crate::event::Event;
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
use serde::Deserialize;
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
    -- TODO: Re-add sequence number when it's used
    -- sequence_number bigint not null,
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
    domain: String,
}

impl PgStoreAdapter {
    /// Create a new Postgres store
    ///
    /// This will attempt to create the events table and indexes if they do not already exist
    pub async fn new(
        conn: Pool<PostgresConnectionManager>,
        domain: String,
    ) -> Result<Self, io::Error> {
        conn.get()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
            .batch_execute(INIT_QUERIES)?;

        Ok(Self { conn, domain })
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
    pub fn last_event<ED>(&self) -> Result<Option<LastHandledEvent>, io::Error>
    where
        ED: EventData,
    {
        let rows = self
            .conn
            .get()
            .unwrap()
            .query(
                r#"select * from last_handled_event_log
                    where event_namespace = $1
                    and event_type = $2
                    and domain = $3
                    limit 1"#,
                &[&ED::event_namespace(), &ED::event_type(), &self.domain],
            )
            .expect("Unable to query database (last_event)");

        if rows.len() == 1 {
            let row = rows.get(0);

            Ok(Some(LastHandledEvent {
                domain: row.get("domain"),
                event_namespace: row.get("event_namespace"),
                event_type: row.get("event_type"),
                event_id: row.get("event_id"),
                time: row.get("time"),
                sequence_number: row.get("sequence_number"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Fetch events of a given type starting from a timestamp going forward
    pub async fn read_events_since<'a, ED>(
        &'a self,
        since: DateTime<Utc>,
    ) -> Result<Vec<Event<ED>>, io::Error>
    where
        ED: EventData + for<'de> Deserialize<'de>,
    {
        let event_type = ED::event_type();
        let event_namespace = ED::event_namespace();

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

                let evt: Event<ED> = serde_json::from_value(json!({
                    "id": id,
                    "data": data_json,
                    "context": context_json,
                }))
                .unwrap();

                evt
            })
            .collect()
            .expect("Failed to collect results");

        trans.finish().expect("Could not finish transaction");

        Ok(results)
    }

    /// Read raw events since a time
    pub async fn read_raw_events_since<'a>(&'a self, event_namespace: &'a str, event_type: &'a str, since: DateTime<Utc>) -> Result<Vec<JsonValue>, io::Error> {
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

    /// Check whether an event exists for a given event ID
    pub fn event_exists<'a>(&'a self, event_id: &'a Uuid) -> Result<bool, io::Error> {
        let rows = self
            .conn
            .get()
            .unwrap()
            .query("select * from events where id = $1 limit 1", &[event_id])?;

        debug_assert!(rows.len() == 1, "Event existence check returned more than one row. This should not be possibe; events MUST have unique IDs.");

        Ok(rows.len() == 1)
    }

    /// Update latest event handled time
    pub fn update_last_handled_event_log<ED>(&self, event: &Event<ED>) -> Result<(), io::Error>
    where
        ED: EventData,
    {
        self.conn
            .get()
            .unwrap()
            .execute(
                r#"insert into last_handled_event_log
                    (domain, event_namespace, event_type, event_id, time)
                    values
                    ($1, $2, $3, $4, $5)
                    on conflict (domain, event_namespace, event_type) do update
                    set event_id = excluded.event_id,
                    time = excluded.time"#,
                &[
                    &self.domain,
                    &ED::event_namespace(),
                    &ED::event_type(),
                    &event.id,
                    &event.context.time
                ],
            )
            .map(|_| ())
            .map_err(|e| io::Error::new(
                io::ErrorKind::Other,
                e.to_string()
            ))
    }
}
