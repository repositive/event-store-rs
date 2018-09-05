extern crate event_store;
extern crate r2d2;
extern crate r2d2_postgres;

use event_store::testhelpers::{
    TestCounterEntity, TestDecrementEvent, TestEvents, TestIncrementEvent,
};
use event_store::{
    adapters::{PgCacheAdapter, PgStoreAdapter, StubEmitterAdapter},
    Aggregator, Event, EventStore, Store,
};
use r2d2::PooledConnection;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use std::thread;
use std::time::Duration;

fn connect() -> PooledConnection<PostgresConnectionManager> {
    let manager = PostgresConnectionManager::new(
        "postgres://postgres@localhost:5430/eventstorerust",
        TlsMode::None,
    ).unwrap();

    let pool = r2d2::Pool::new(manager).unwrap();

    pool.get().expect("Could not connect to Postgres DB")
}

#[test]
fn it_aggregates_events() {
    let events = vec![
        Event::from_data(TestEvents::Inc(TestIncrementEvent {
            by: 1,
            ident: "it_aggregates_events".into(),
        })),
        Event::from_data(TestEvents::Inc(TestIncrementEvent {
            by: 1,
            ident: "it_aggregates_events".into(),
        })),
        Event::from_data(TestEvents::Dec(TestDecrementEvent {
            by: 2,
            ident: "it_aggregates_events".into(),
        })),
        Event::from_data(TestEvents::Inc(TestIncrementEvent {
            by: 2,
            ident: "it_aggregates_events".into(),
        })),
        Event::from_data(TestEvents::Dec(TestDecrementEvent {
            by: 3,
            ident: "it_aggregates_events".into(),
        })),
        Event::from_data(TestEvents::Dec(TestDecrementEvent {
            by: 3,
            ident: "it_aggregates_events".into(),
        })),
    ];

    let result: TestCounterEntity = events
        .iter()
        .fold(TestCounterEntity::default(), TestCounterEntity::apply_event);

    assert_eq!(result, TestCounterEntity { counter: -4 });
}

#[test]
fn it_queries_the_database() {
    let conn = connect();

    let store_adapter = PgStoreAdapter::new(&conn);
    let cache_adapter = PgCacheAdapter::new(&conn);
    let emitter_adapter = StubEmitterAdapter::new();

    let store = EventStore::new(store_adapter, cache_adapter, emitter_adapter);

    let ident = String::from("dbquery");

    assert!(
        store
            .save(Event::from_data(TestEvents::Inc(TestIncrementEvent {
                by: 99,
                ident: ident.clone()
            }))).is_ok()
    );

    let entity: TestCounterEntity = store.aggregate(ident).unwrap();

    assert_eq!(entity.counter, 99);
}

#[test]
fn it_saves_events() {
    let conn = connect();

    let store_adapter = PgStoreAdapter::new(&conn);
    let cache_adapter = PgCacheAdapter::new(&conn);
    let emitter_adapter = StubEmitterAdapter::new();

    let store = EventStore::new(store_adapter, cache_adapter, emitter_adapter);

    let event = TestEvents::Inc(TestIncrementEvent {
        by: 123123,
        ident: "it_saves_events".into(),
    });

    assert!(store.save(Event::from_data(event)).is_ok());
}

#[test]
fn it_uses_the_aggregate_cache() {
    let conn = connect();

    let ident = "aggregatecache";

    conn.execute("DELETE FROM events WHERE data->>'ident' = $1", &[&ident])
        .expect("Truncate");
    conn.execute("TRUNCATE aggregate_cache", &[])
        .expect("Truncate");

    let store_adapter = PgStoreAdapter::new(&conn);
    let cache_adapter = PgCacheAdapter::new(&conn);
    let emitter_adapter = StubEmitterAdapter::new();

    let store = EventStore::new(store_adapter, cache_adapter, emitter_adapter);

    assert!(
        store
            .save(Event::from_data(TestEvents::Inc(TestIncrementEvent {
                by: 1,
                ident: ident.into()
            }))).is_ok()
    );

    assert!(
        store
            .save(Event::from_data(TestEvents::Inc(TestIncrementEvent {
                by: 2,
                ident: ident.into()
            }))).is_ok()
    );

    // Wait for DB to process
    thread::sleep(Duration::from_millis(10));

    let entity: TestCounterEntity = store.aggregate(ident.into()).unwrap();

    assert_eq!(entity.counter, 3);
}
