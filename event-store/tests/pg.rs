#[macro_use]
extern crate event_store;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate tokio;

use event_store::prelude::*;
use event_store::testhelpers::{pg_connect, TestCounterEntity, TestIncrementEvent};
use event_store::{
    adapters::{PgCacheAdapter, PgStoreAdapter, StubEmitterAdapter},
    Event, EventStore,
};

use std::thread;
use std::time::Duration;

#[test]
fn it_queries_the_database() {
    let conn = pg_connect();
    let store = pg_store!(conn);
    let ident = String::from("it_queries_the_database");

    pg_delete_events!(conn, ident);

    let test_event = Event::from_data(TestIncrementEvent {
        by: 99,
        ident: ident.clone(),
    });

    assert!(store.save(&test_event).is_ok());

    let entity: TestCounterEntity = store.aggregate(ident).unwrap();

    assert_eq!(entity.counter, 99);
}

#[test]
fn it_saves_events() {
    let conn = pg_connect();
    let store = pg_store!(conn);
    let ident = String::from("it_saves_events");

    pg_delete_events!(conn, ident);

    let event = Event::from_data(TestIncrementEvent {
        by: 123123,
        ident: ident.clone(),
    });

    assert!(store.save(&event).is_ok());
}

#[test]
fn it_uses_the_aggregate_cache() {
    let conn = pg_connect();
    let store = pg_store!(conn);
    let ident = String::from("it_uses_the_aggregate_cache");

    pg_delete_events!(conn, ident);
    pg_empty_cache!(conn);

    let test_ev_1 = Event::from_data(TestIncrementEvent {
        by: 1,
        ident: ident.clone(),
    });

    let test_ev_2 = Event::from_data(TestIncrementEvent {
        by: 2,
        ident: ident.clone(),
    });

    assert!(store.save(&test_ev_1).is_ok());
    assert!(store.save(&test_ev_2).is_ok());

    // Wait for DB to process
    thread::sleep(Duration::from_millis(10));

    let entity: TestCounterEntity = store.aggregate(ident).unwrap();

    assert_eq!(entity.counter, 3);
}
