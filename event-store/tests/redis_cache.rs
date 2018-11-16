#[macro_use]
extern crate event_store;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate tokio;

use event_store::prelude::*;
use event_store::testhelpers::{pg_connect, redis_connect, TestCounterEntity, TestIncrementEvent};
use event_store::{
    adapters::{PgStoreAdapter, RedisCacheAdapter, StubEmitterAdapter},
    Event, EventStore,
};

use std::thread;
use std::time::Duration;

#[test]
fn redis_aggregate_cache() {
    let conn = pg_connect();
    let redis_conn = redis_connect();

    redis_empty_cache!(redis_conn);

    let store = pg_store_with_redis_cache!(conn, redis_conn);
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
