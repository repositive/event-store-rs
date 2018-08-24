//! Store adapter backed by Postgres

use adapters::pg::PgQuery;
use adapters::{CacheResult, StoreAdapter};
use chrono::prelude::*;
use fallible_iterator::FallibleIterator;
use postgres::types::ToSql;
use postgres::Connection;
use serde::Serialize;
use serde_json::{from_value, to_value, Value as JsonValue};
use std::marker::PhantomData;
use uuid::Uuid;
use Aggregator;
use EventContext;
use Events;

/// Postgres store adapter
pub struct PgStoreAdapter<'a, E> {
    phantom: PhantomData<E>,
    conn: &'a Connection,
}

impl<'a, E> PgStoreAdapter<'a, E> {
    /// Create a new PgStore from a Postgres DB connection
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            conn,
            phantom: PhantomData,
        }
    }
}

impl<'a, E> StoreAdapter<E, PgQuery<'a>> for PgStoreAdapter<'a, E>
where
    E: Events,
{
    fn aggregate<T, A>(&self, query_args: A, since: Option<CacheResult<T>>) -> Result<T, String>
    where
        T: Aggregator<E, A, PgQuery<'a>> + Default,
        A: Clone,
    {
        let q = T::query(query_args);

        let (initial_state, query_string) = if let Some((existing, time)) = since {
            (existing, format!(
                "SELECT * FROM ({}) AS events WHERE events.context->>'time' >= '{}' ORDER BY events.context->>'time' ASC",
                q.query, time
            ))
        } else {
            (T::default(), String::from(q.query))
        };

        let trans = self.conn.transaction().expect("Tranny");
        let stmt = trans.prepare(&query_string).expect("Prep");

        let mut params: Vec<&ToSql> = Vec::new();

        for (i, _arg) in q.args.iter().enumerate() {
            params.push(&*q.args[i]);
        }

        let results = stmt
            .lazy_query(&trans, &params, 1000)
            .expect("Query")
            .map(|row| {
                let json: JsonValue = row.get("data");
                let evt: E = from_value(json).expect("Decode");

                evt
            }).fold(initial_state, |acc, event| T::apply_event(acc, &event))
            .expect("Fold");

        trans.finish().expect("Tranny finished");

        Ok(results)
    }

    fn save<S>(&self, event: &E, subject: Option<S>) -> Result<(), String>
    where
        S: Serialize,
    {
        let time: DateTime<Utc> = Utc::now();
        let context = EventContext {
            action: None,
            subject: subject.map(|s| to_value(s).expect("Could not serialize subject")),
            time,
        };
        let id = Uuid::new_v4();

        self.conn
            .execute(
                r#"INSERT INTO events (id, data, context)
                VALUES ($1, $2, $3)"#,
                &[
                    &id,
                    &to_value(event).expect("Item to value"),
                    &to_value(context).expect("Context to value"),
                ],
            ).expect("Save");

        Ok(())
    }
}
