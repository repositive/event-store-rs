use crate::aggregator::Aggregator;
use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::pg::PgQuery;
use crate::store::Store;
use event_store_derive::*;
use log::trace;
use postgres::types::ToSql;
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use serde_derive::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// Set of all events in the domain
#[derive(Events, Debug)]
pub enum TestEvents {
    Inc(Event<TestEvent>),
}

#[derive(EventData, Debug)]
#[event_store(namespace = "some_namespace")]
pub struct TestEvent {
    pub num: i32,
}

/// Testing entity for a pretend domain
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct TestCounterEntity {
    /// Current counter value
    pub counter: i32,
}

impl Default for TestCounterEntity {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

impl Aggregator<TestEvents, String, PgQuery> for TestCounterEntity {
    fn apply_event(acc: Self, event: &TestEvents) -> Self {
        let counter = match event {
            TestEvents::Inc(ref inc) => acc.counter + inc.data.num,
        };

        Self { counter, ..acc }
    }

    fn query(_query_args: String) -> PgQuery {
        let params: Vec<Box<ToSql + Send + Sync>> = Vec::new();

        PgQuery::new("select * from events", params)
    }
}

impl EventHandler for TestEvent {
    fn handle_event(event: Event<Self>, _store: &Store) {
        trace!("Handle event {:?}", event);
    }
}

fn current_time_ms() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let in_ms =
        since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

    in_ms
}

/// Create a new database with a random name, returning the connection
pub fn pg_create_random_db() -> Pool<PostgresConnectionManager> {
    let db_id = format!("eventstorerust-test-{}", current_time_ms());

    println!("Create test DB {}", db_id);

    let manager =
        PostgresConnectionManager::new("postgres://postgres@localhost:5430", TlsMode::None)
            .unwrap();

    let pool = r2d2::Pool::new(manager).unwrap();

    let conn = pool.get().unwrap();

    conn.batch_execute(&format!("CREATE DATABASE \"{}\"", db_id))
        .unwrap();

    let manager = PostgresConnectionManager::new(
        format!("postgres://postgres@localhost:5430/{}", db_id),
        TlsMode::None,
    )
    .unwrap();

    let pool = r2d2::Pool::new(manager).unwrap();

    let conn = pool.get().unwrap();

    conn.batch_execute(
        r#"
        CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

        CREATE TABLE events (
            id uuid DEFAULT uuid_generate_v4() PRIMARY KEY,
            data jsonb NOT NULL,
            context jsonb DEFAULT '{}'::jsonb
        );

        CREATE TABLE aggregate_cache (
            id VARCHAR(64) PRIMARY KEY,
            data jsonb NOT NULL,
            time timestamp without time zone DEFAULT now()
        );
    "#,
    )
    .unwrap();

    pool
}
