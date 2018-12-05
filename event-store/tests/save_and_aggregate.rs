#[macro_use]
extern crate log;
extern crate event_store;
extern crate pretty_env_logger;

use event_store::*;
use futures::future;
use futures::prelude::*;
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_core::reactor::Core;

#[test]
fn save_and_aggregate() {
    pretty_env_logger::init();

    let test_event = TestEvent { num: 100 };
    let test_event_2 = TestEvent { num: 200 };

    trace!("Save and emit test");

    let pool = pg_create_random_db();

    let conn = pool.get().unwrap();

    let event_saver = EventSaver::new(pool.clone());

    let mut core = Core::new().unwrap();

    let run = event_saver
        .save(&Event::from_data(test_event))
        .join(event_saver.save(&Event::from_data(test_event_2)))
        .and_then(|_| pg_read(conn, TestCounterEntity::query(String::new()), None))
        .and_then(|events: Vec<TestEvents>| {
            future::ok(
                events
                    .iter()
                    .fold(TestCounterEntity::default(), TestCounterEntity::apply_event),
            )
        })
        .and_then(|aggregate| {
            info!("Aggregate result {:?}", aggregate);

            future::ok(())
        })
        .map_err(|e| {
            error!("Run error: {}", e);

            ()
        });

    core.run(run).unwrap();
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
