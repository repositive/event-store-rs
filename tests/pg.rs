extern crate event_store_rs;
extern crate postgres;

use event_store_rs::testhelpers::{
    TestCounterEntity, TestDecrementEvent, TestEvents, TestIncrementEvent,
};
use event_store_rs::{Aggregator, PgStore, Store};
use postgres::{Connection, TlsMode};

#[test]
fn it_aggregates_events() {
    let events = vec![
        TestEvents::Inc(TestIncrementEvent { by: 1 }),
        TestEvents::Inc(TestIncrementEvent { by: 1 }),
        TestEvents::Dec(TestDecrementEvent { by: 2 }),
        TestEvents::Inc(TestIncrementEvent { by: 2 }),
        TestEvents::Dec(TestDecrementEvent { by: 3 }),
        TestEvents::Dec(TestDecrementEvent { by: 3 }),
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

    let _entity: TestCounterEntity = store.aggregate("inc_dec".into());
}
