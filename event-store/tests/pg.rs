#[macro_use]
extern crate event_store;
extern crate env_logger;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate tokio;
#[macro_use]
extern crate log;
extern crate futures;

use event_store::prelude::*;
use event_store::testhelpers::{pg_create_random_db, TestCounterEntity, TestIncrementEvent};
use event_store::{
    adapters::{PgCacheAdapter, PgStoreAdapter, StubEmitterAdapter},
    Event, EventStore,
};
use futures::future::ok as FutOk;
use futures::lazy;
use futures::Future;
use tokio::runtime::current_thread::Runtime as CurrentThreadRuntime;

#[test]
fn it_queries_the_database() {
    env_logger::init();

    let conn = pg_create_random_db("query");

    let ident = String::from("it_queries_the_database");

    pg_delete_events!(conn, ident);

    let test_event = Event::from_data(TestIncrementEvent {
        by: 99,
        ident: ident.clone(),
    });

    let fut = FutOk(())
        .and_then(|_| {
            let store = pg_store!(conn);

            store.save(&test_event).map(|_| store)
        })
        .and_then(|store| {
            trace!("Both events saved");

            let entity: TestCounterEntity = store.aggregate(ident).unwrap();

            FutOk(entity)
        });

    let mut rt = CurrentThreadRuntime::new().unwrap();

    let result = rt.block_on(fut).unwrap();

    assert_eq!(result, TestCounterEntity { counter: 99 });
}

#[test]
fn it_saves_events() {
    let conn = pg_create_random_db("save");
    let ident = String::from("it_saves_events");

    pg_delete_events!(conn, ident);

    let event = Event::from_data(TestIncrementEvent { by: 123123, ident });

    let fut = lazy(|| {
        let store = pg_store!(conn);

        store.save(&event).map(|_| store)
    });

    let mut rt = CurrentThreadRuntime::new().unwrap();

    rt.block_on(fut).unwrap();
}

#[test]
fn it_aggregates_multiple_events() {
    let conn = pg_create_random_db("aggregate");
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
        let store = pg_store!(conn);

        store
            .save(&test_ev_1)
            .join(store.save(&test_ev_2))
            .map(|_| store)
    })
    .and_then(|store| {
        trace!("Both events saved");

        let entity: TestCounterEntity = store.aggregate(ident).unwrap();

        assert_eq!(entity.counter, 3);

        FutOk(())
    });

    let mut rt = CurrentThreadRuntime::new().unwrap();

    rt.block_on(fut).unwrap();
}
