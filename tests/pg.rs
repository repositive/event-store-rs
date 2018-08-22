extern crate event_store_rs;
extern crate postgres;

use event_store_rs::testhelpers::{
    TestCounterEntity, TestDecrementEvent, TestEvents, TestIncrementEvent,
};
use event_store_rs::{pg::PgStore, Aggregator, Store};
use postgres::{Connection, TlsMode};
use std::thread;
use std::time::Duration;

#[test]
fn it_aggregates_events() {
    let events = vec![
        TestEvents::Inc(TestIncrementEvent {
            by: 1,
            ident: "it_aggregates_events".into(),
        }),
        TestEvents::Inc(TestIncrementEvent {
            by: 1,
            ident: "it_aggregates_events".into(),
        }),
        TestEvents::Dec(TestDecrementEvent {
            by: 2,
            ident: "it_aggregates_events".into(),
        }),
        TestEvents::Inc(TestIncrementEvent {
            by: 2,
            ident: "it_aggregates_events".into(),
        }),
        TestEvents::Dec(TestDecrementEvent {
            by: 3,
            ident: "it_aggregates_events".into(),
        }),
        TestEvents::Dec(TestDecrementEvent {
            by: 3,
            ident: "it_aggregates_events".into(),
        }),
    ];

    let result: TestCounterEntity = events
        .iter()
        .fold(TestCounterEntity::default(), TestCounterEntity::apply_event);

    assert_eq!(result, TestCounterEntity { counter: -4 });
}

#[test]
fn it_queries_the_database() {
    let conn = Connection::connect(
        "postgres://postgres@localhost:5430/eventstorerust",
        TlsMode::None,
    ).expect("Could not connect to DB");

    let store = PgStore::new(conn);

    let ident = String::from("dbquery");

    assert!(
        store
            .save(
                TestEvents::Inc(TestIncrementEvent {
                    by: 99,
                    ident: ident.clone()
                }),
                None::<()>
            ).is_ok()
    );

    let entity: TestCounterEntity = store.aggregate(ident);

    assert_eq!(entity.counter, 99);
}

#[test]
fn it_saves_events() {
    let conn = Connection::connect(
        "postgres://postgres@localhost:5430/eventstorerust",
        TlsMode::None,
    ).expect("Could not connect to DB");

    let store = PgStore::<TestEvents>::new(conn);

    let event = TestEvents::Inc(TestIncrementEvent {
        by: 123123,
        ident: "it_saves_events".into(),
    });

    assert!(store.save(event, None::<()>).is_ok());
}

#[test]
fn it_uses_the_aggregate_cache() {
    let conn = Connection::connect(
        "postgres://postgres@localhost:5430/eventstorerust",
        TlsMode::None,
    ).expect("Could not connect to DB");

    let ident = "aggregatecache";

    conn.execute("DELETE FROM events WHERE data->>'ident' = $1", &[&ident])
        .expect("Truncate");
    conn.execute("TRUNCATE aggregate_cache", &[])
        .expect("Truncate");

    let store = PgStore::<TestEvents>::new(conn);

    assert!(
        store
            .save(
                TestEvents::Inc(TestIncrementEvent {
                    by: 1,
                    ident: ident.into()
                }),
                None::<()>
            ).is_ok()
    );

    assert!(
        store
            .save(
                TestEvents::Inc(TestIncrementEvent {
                    by: 2,
                    ident: ident.into()
                }),
                None::<()>
            ).is_ok()
    );

    // Wait for DB to process
    thread::sleep(Duration::from_millis(10));

    let entity: TestCounterEntity = store.aggregate(ident.into());

    assert_eq!(entity.counter, 3);
}
