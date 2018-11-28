#[macro_use]
extern crate event_store;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate tokio;
#[macro_use]
extern crate log;
extern crate futures;

use event_store::prelude::*;
use event_store::testhelpers::{
    pg_create_random_db, redis_connect, TestCounterEntity, TestIncrementEvent,
};
use event_store::{
    adapters::{PgStoreAdapter, RedisCacheAdapter, StubEmitterAdapter},
    Event, EventStore,
};
use futures::future::ok as FutOk;
use futures::lazy;
use futures::Future;
use tokio::runtime::Runtime;

#[test]
fn redis_aggregate_cache() {
    let conn = pg_create_random_db("redis");
    let redis_conn = redis_connect();

    redis_empty_cache!(redis_conn);

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

    let fut = lazy(move || {
        let store = pg_store_with_redis_cache!(conn, redis_conn);

        store
            .save(&test_ev_1)
            .join(store.save(&test_ev_2))
            .map(|_| store)
    })
    .and_then(|store| {
        trace!("Both events saved");

        store.aggregate(ident)
    })
    .and_then(|entity: TestCounterEntity| {
        assert_eq!(entity.counter, 3);

        FutOk(())
    });

    let rt = Runtime::new().unwrap();

    rt.block_on_all(fut).unwrap();
}
