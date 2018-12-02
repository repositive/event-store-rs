extern crate event_store;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate futures;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate tokio;

use event_store::prelude::*;
use event_store::testhelpers::{
    amqp_clear_queue, pg_create_random_db, TestCounterEntity, TestIncrementEvent,
};
use event_store::{
    adapters::{AMQPEmitterAdapter, AMQPEmitterOptions, PgCacheAdapter, PgStoreAdapter},
    Event, EventStore,
};
use futures::future::ok as FutOk;
use futures::lazy;
use futures::Future;
use std::io;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::timer::Delay;

#[test]
fn event_replay_all_events() {
    env_logger::init();

    let other_conn = pg_create_random_db("replay-other");
    let this_conn = pg_create_random_db("replay-this");

    let ident = String::from("event_replay");
    let _ident = ident.clone();

    let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();

    let amqp = AMQPEmitterAdapter::new(AMQPEmitterOptions {
        uri: addr,
        exchange: "_test".into(),
        namespace: "this-store",
    });
    let other_amqp = AMQPEmitterAdapter::new(AMQPEmitterOptions {
        uri: addr,
        exchange: "_test".into(),
        namespace: "other-store",
    });

    let store_adapter = PgStoreAdapter::new(this_conn.clone());
    let cache_adapter = PgCacheAdapter::new(this_conn.clone());

    let other_store_adapter = PgStoreAdapter::new(other_conn.clone());
    let other_cache_adapter = PgCacheAdapter::new(other_conn.clone());

    let fut = lazy(|| {
        let other_store = EventStore::new(other_store_adapter, other_cache_adapter, other_amqp);

        other_store
            .save(&Event::from_data(TestIncrementEvent {
                by: 1,
                ident: ident.clone(),
            }))
            .join3(
                other_store.save(&Event::from_data(TestIncrementEvent {
                    by: 2,
                    ident: ident.clone(),
                })),
                other_store.save(&Event::from_data(TestIncrementEvent { by: 3, ident })),
            )

        // FutOk(())
    })
    .and_then(|_| {
        trace!("Clear queue other-store-some_namespace.TestIncrementEvent");
        amqp_clear_queue("other-store-some_namespace.TestIncrementEvent")
    })
    // This queue doesn't normally exist on an initial run, but if it does (from a previous test)
    // run, it should be emptied so the receiving store does not attempt to consume duplicated
    // events.
    .and_then(|_| {
        trace!("Clear queue this-store-some_namespace.TestIncrementEvent");
        amqp_clear_queue("this-store-some_namespace.TestIncrementEvent")
    })
    .and_then(|_| {
        let this_store = EventStore::new(store_adapter, cache_adapter, amqp);

        trace!("This store initialised");

        this_store
            .subscribe(|_evt: Event<TestIncrementEvent>, _| {
                info!("Event handler for {:?}", _evt);
            })
            .map(|_| this_store)
    })
    .and_then(|this_store| {
        amqp_clear_queue("this-store-some_namespace.TestIncrementEvent").map(|_| this_store)
    })
    .and_then(|store| {
        // Wait for queues to settle
        Delay::new(Instant::now() + Duration::from_millis(100))
            .map(|_| store)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "wait error"))
    })
    .and_then(|store| {
        trace!("Event saved");

        store.aggregate(_ident)
    })
    .and_then(|entity: TestCounterEntity| FutOk(entity));

    let mut rt = Runtime::new().unwrap();

    let entity: TestCounterEntity = rt.block_on(fut).unwrap();

    assert_eq!(entity.counter, 6);
}
