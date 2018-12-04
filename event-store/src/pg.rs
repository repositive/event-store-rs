use crate::event::Event;
use event_store_derive_internals::EventData;
use futures::future;
use futures::future::IntoFuture;
use futures::stream::Stream;
use futures::Future;
use postgres::Connection;
use r2d2::{self, Pool, PooledConnection};
use r2d2_postgres::postgres::types::ToSql;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use serde_json::to_value;
use std::io::{self, ErrorKind};
use std::net::SocketAddr;
use std::str;
use tokio::net::TcpStream;
use tokio_core::reactor::Core;

/// Connect to a local Postgres database on port 5430
pub fn pg_connect() -> Pool<PostgresConnectionManager> {
    let manager = PostgresConnectionManager::new(
        "postgres://postgres@localhost:5430/eventstorerust",
        TlsMode::None,
    )
    .unwrap();

    let pool = r2d2::Pool::new(manager).unwrap();

    pool
}

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
