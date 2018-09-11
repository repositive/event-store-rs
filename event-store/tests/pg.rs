extern crate event_store;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate tokio;

use event_store::prelude::*;
use event_store::testhelpers::{TestCounterEntity, TestIncrementEvent};
use event_store::{
    adapters::{PgCacheAdapter, PgStoreAdapter, StubEmitterAdapter},
    Event, EventStore,
};
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use std::thread;
use std::time::Duration;
use tokio::runtime::current_thread::block_on_all;

fn connect() -> Pool<PostgresConnectionManager> {
    let manager = PostgresConnectionManager::new(
        "postgres://postgres@localhost:5430/eventstorerust",
        TlsMode::None,
    ).unwrap();

    let pool = r2d2::Pool::new(manager).unwrap();

    pool
}

#[test]
fn it_queries_the_database() {
    let conn = connect();

    let store_adapter = PgStoreAdapter::new(conn.clone());
    let cache_adapter = PgCacheAdapter::new(conn.clone());
    let emitter_adapter = StubEmitterAdapter::new();

    let store = EventStore::new(store_adapter, cache_adapter, emitter_adapter);

    let ident = String::from("dbquery");

    assert!(
        block_on_all(store.save(Event::from_data(TestIncrementEvent {
            by: 99,
            ident: ident.clone()
        }))).is_ok()
    );

    let entity: TestCounterEntity = block_on_all(store.aggregate(ident)).unwrap();

    assert_eq!(entity.counter, 99);
}

#[test]
fn it_saves_events() {
    let conn = connect();

    let store_adapter = PgStoreAdapter::new(conn.clone());
    let cache_adapter = PgCacheAdapter::new(conn.clone());
    let emitter_adapter = StubEmitterAdapter::new();

    let store = EventStore::new(store_adapter, cache_adapter, emitter_adapter);

    let event = TestIncrementEvent {
        by: 123123,
        ident: "it_saves_events".into(),
    };

    assert!(block_on_all(store.save(Event::from_data(event))).is_ok());
}

#[test]
fn it_uses_the_aggregate_cache() {
    let conn = connect();

    let ident = "aggregatecache";

    conn.get()
        .unwrap()
        .execute("DELETE FROM events WHERE data->>'ident' = $1", &[&ident])
        .expect("Truncate");
    conn.get()
        .unwrap()
        .execute("TRUNCATE aggregate_cache", &[])
        .expect("Truncate");

    let store_adapter = PgStoreAdapter::new(conn.clone());
    let cache_adapter = PgCacheAdapter::new(conn.clone());
    let emitter_adapter = StubEmitterAdapter::new();

    let store = EventStore::new(store_adapter, cache_adapter, emitter_adapter);

    assert!(
        block_on_all(store.save(Event::from_data(TestIncrementEvent {
            by: 1,
            ident: ident.into()
        }))).is_ok()
    );

    assert!(
        block_on_all(store.save(Event::from_data(TestIncrementEvent {
            by: 2,
            ident: ident.into()
        }))).is_ok()
    );

    // Wait for DB to process
    thread::sleep(Duration::from_millis(10));

    let entity: TestCounterEntity = block_on_all(store.aggregate(ident.into())).unwrap();

    assert_eq!(entity.counter, 3);
}
