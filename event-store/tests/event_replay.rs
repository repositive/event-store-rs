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
    adapters::{AMQPEmitterAdapter, PgCacheAdapter, PgStoreAdapter},
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

    let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();

    let amqp = AMQPEmitterAdapter::new(addr, "_test".into());
    let other_amqp = AMQPEmitterAdapter::new(addr, "_test".into());

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
            .map(|_| (other_store, ident))
            .and_then(move |(store, ident)| {
                store
                    .save(&Event::from_data(TestIncrementEvent {
                        by: 2,
                        ident: ident.clone(),
                    }))
                    .map(|_| (store, ident))
            })
            .and_then(move |(store, ident)| {
                store
                    .save(&Event::from_data(TestIncrementEvent { by: 3, ident }))
                    .map(|_| store)
            })
    })
    .and_then(|_| amqp_clear_queue("some_namespace.TestIncrementEvent"))
    .and_then(|_| {
        Delay::new(Instant::now() + Duration::from_millis(100))
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "wait error"))
    })
    .and_then(|_| {
        trace!("This store initialised");

        let this_store = EventStore::new(store_adapter, cache_adapter, amqp);

        this_store
            .subscribe(|_evt: Event<TestIncrementEvent>, _| {
                info!("Event handler for {:?}", _evt);
            })
            .map(|_| this_store)
    })
    .and_then(|store| {
        Delay::new(Instant::now() + Duration::from_millis(100))
            .map(|_| store)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "wait error"))
    })
    .and_then(move |this_store| {
        let entity: TestCounterEntity = this_store.aggregate("event_replay".into()).unwrap();

        debug!("Aggregated entity: {:?}", entity);

        assert_eq!(entity.counter, 6);

        FutOk(this_store)
    });

    let mut rt = Runtime::new().unwrap();

    rt.block_on(fut).unwrap();
}
