use crate::event::Event;
use crate::pg::PgQuery;
use chrono::prelude::*;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use fallible_iterator::FallibleIterator;
use futures::future;
use futures::Future;
use r2d2::{self, PooledConnection};
use r2d2_postgres::postgres::types::ToSql;
use r2d2_postgres::PostgresConnectionManager;
use serde_json::to_value;
use serde_json::{from_value, Value as JsonValue};
use std::io;
use uuid::Uuid;

/// Save an event into PG
pub fn pg_save<ED>(
    conn: PooledConnection<PostgresConnectionManager>,
    event: &Event<ED>,
) -> impl Future<Item = (), Error = io::Error>
where
    ED: EventData,
{
    debug!(
        "Insert event {}.{}",
        ED::event_namespace(),
        ED::event_type()
    );

    conn.prepare("insert into events (id, data, context) values ($1, $2, $3)")
        .and_then(|stmt| {
            stmt.execute(&[
                &event.id,
                &to_value(&event.data).expect("Unable to convert event data to value"),
                &to_value(&event.context).expect("Cannot convert event context"),
            ])
        })
        .map(|_| future::ok(()))
        .unwrap_or_else(|_| future::err(io::Error::new(io::ErrorKind::Other, "Could not save")))
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

/// Read a list of events
pub fn pg_read<E>(
    conn: PooledConnection<PostgresConnectionManager>,
    query: PgQuery,
    since: Option<DateTime<Utc>>,
) -> impl Future<Item = Vec<E>, Error = io::Error>
where
    E: Events,
{
    let query_string = generate_query(&query, since);

    debug!("Read query {}", query_string);

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
        .expect("ain't no collec");

    trans.finish().expect("Could not finish transaction");

    Box::new(future::ok(results))
}
