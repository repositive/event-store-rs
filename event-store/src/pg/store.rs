use crate::event::Event;
use crate::event_context::EventContext;
use crate::pg::PgQuery;
use chrono::prelude::*;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use fallible_iterator::FallibleIterator;
use log::debug;
use r2d2::{self, PooledConnection};
use r2d2_postgres::postgres::types::ToSql;
use r2d2_postgres::PostgresConnectionManager;
use serde_json::{from_value, json, to_value, Value as JsonValue};
use std::io;
use uuid::Uuid;

/// Save an event into PG
pub async fn pg_save<ED>(
    conn: PooledConnection<PostgresConnectionManager>,
    event: &Event<ED>,
) -> Result<(), io::Error>
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
        .map(|_| ())
        .map_err(|_e| io::Error::new(io::ErrorKind::Other, "Could not save"))
    // .unwrap_or_else(|_| future::err(io::Error::new(io::ErrorKind::Other, "Could not save")))
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
pub async fn pg_read<E>(
    conn: PooledConnection<PostgresConnectionManager>,
    query: &PgQuery,
    since: Option<DateTime<Utc>>,
) -> Result<Vec<E>, io::Error>
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

    Ok(results)
}

pub fn pg_last_event<ED>(
    conn: PooledConnection<PostgresConnectionManager>,
) -> Result<Option<Event<ED>>, io::Error>
where
    ED: EventData,
{
    let rows = conn
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
