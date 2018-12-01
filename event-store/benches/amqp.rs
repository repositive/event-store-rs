//! This benchmark run attempts to exercise the store and cache by:
//!
//! * Emptying the database
//! * Saving 10 events
//! * Aggregating (no cache)
//! * Aggregating again (now there is a cache)
//! * Saving 10 more events
//! * Aggregating again (cached entry plus 10 new events)
//! * Aggregating again (cache only)
//!
//! It requires a running Postgres database and Redis instance. You can start them using
//! `docker-compose up` in the project repo.

#[macro_use]
extern crate criterion;
#[macro_use]
extern crate event_store;
extern crate futures;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate tokio;

use criterion::Criterion;
use event_store::testhelpers::TestIncrementEvent;
use event_store::testhelpers::{pg_create_random_db, redis_connect, TestCounterEntity};
use event_store::Event;
use event_store::{
    adapters::{
        AMQPEmitterAdapter, AMQPEmitterOptions, EmitterAdapter, PgCacheAdapter, PgStoreAdapter,
        RedisCacheAdapter, StubEmitterAdapter,
    },
    EventStore,
};
use futures::future::ok as FutOk;
use futures::lazy;
use futures::Future;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::prelude::future::join_all;
use tokio::prelude::*;
use tokio::runtime::Runtime;

fn emit_100_receive_100(c: &mut Criterion) {
    // let conn = pg_create_random_db("exercise-pg");
    let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();

    let amqp = AMQPEmitterAdapter::new(AMQPEmitterOptions {
        uri: addr,
        exchange: "bench".into(),
        namespace: "bench",
    });

    // Runtime::new()
    //     .unwrap()
    //     .block_on(amqp.emit(&Event::from_data(TestIncrementEvent {
    //         by: 10,
    //         ident: "bench".into(),
    //     })));

    c.bench_function("emit 100 events and receive 100 events", move |b| {
        let mut rt = Runtime::new().unwrap();

        let amqp = AMQPEmitterAdapter::new(AMQPEmitterOptions {
            uri: addr,
            exchange: "bench".into(),
            ..AMQPEmitterOptions::default()
        });

        b.iter_with_setup(
            || {
                // let rt = Runtime::new().unwrap();

                // let amqp = AMQPEmitterAdapter::new(AMQPEmitterOptions {
                //     uri: addr,
                //     exchange: "bench".into(),
                //     ..AMQPEmitterOptions::default()
                // });

                // (rt, amqp)
                ()
            },
            move |_| {
                let mut emits = Vec::new();

                for i in 0..1_i32 {
                    emits.push(Box::new(amqp.emit(&Event::from_data(TestIncrementEvent {
                        by: i,
                        ident: "bench".into(),
                    }))));
                }

                // let send = join_all((0..1_i32).map(move |i| {
                //     Box::new(amqp.emit(&Event::from_data(TestIncrementEvent {
                //         by: i,
                //         ident: "bench".into(),
                //     })))
                // }));

                rt.block_on(join_all(emits)).unwrap();

                // rt.block_on(amqp.emit(&Event::from_data(TestIncrementEvent {
                //     by: 10,
                //     ident: "bench".into(),
                // })))
                // .unwrap();

                // let fut = amqp
                //     .subscribe(move |_event: Event<TestIncrementEvent>| {
                //         &sh.lock().unwrap().send(()).unwrap();
                //     })
                //     .and_then(move |_| {
                //         amqp.emit(&Event::from_data(TestIncrementEvent {
                //             by: 1,
                //             ident: "some_ident".into(),
                //         }))
                //     });
            },
        )
    });
}

criterion_group!(
    name = postgres;

    config = Criterion::default()
        .warm_up_time(Duration::from_millis(1000))
        .sample_size(20)
        .measurement_time(Duration::from_millis(4000));

    targets =
        emit_100_receive_100,
);
criterion_main!(postgres);
