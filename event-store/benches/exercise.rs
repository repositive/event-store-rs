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
use event_store::prelude::*;
use event_store::testhelpers::{pg_create_random_db, redis_connect, TestCounterEntity};
use event_store::{
    adapters::{PgCacheAdapter, PgStoreAdapter, RedisCacheAdapter, StubEmitterAdapter},
    EventStore,
};
use futures::future::ok as FutOk;
use futures::lazy;
use futures::Future;
use std::time::Duration;
use tokio::runtime::Runtime;

fn pg_store_pg_cache(c: &mut Criterion) {
    let conn = pg_create_random_db("exercise-pg");

    c.bench_function("exercise pg store and pg cache", move |b| {
        b.iter_with_setup(
            || {
                pg_empty_events!(conn);
                pg_empty_cache!(conn);
            },
            |_| {
                let _conn = conn.clone();

                let fut = lazy(move || {
                    let store = pg_store!(_conn);

                    FutOk((store, _conn))
                })
                .and_then(move |(store, conn)| {
                    conn.get().unwrap().batch_execute(r#"
                        INSERT INTO "events" ("id", "data", "context") VALUES
                        ('00dda037-21c8-41c4-9b00-39d3ac771af3',    '{"by": 1, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.240194600Z", "action": null, "subject": null}'),
                        ('37c67766-3ed1-4f42-83cb-4105d64bd672',    '{"by": 1, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.257805700Z", "action": null, "subject": null}'),
                        ('a63725e2-fcd6-44f2-8cb7-d7bf22ecf03c',    '{"by": 2, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.260671600Z", "action": null, "subject": null}'),
                        ('9ce4b0cd-f415-4091-b298-99812917dea1',    '{"by": 2, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.263277800Z", "action": null, "subject": null}'),
                        ('a477403d-b8ac-41a4-b5db-e712ee0f0f8b',    '{"by": 3, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.265985Z", "action": null, "subject": null}'),
                        ('c5e6297f-24bf-4f98-a13e-56511cd73a15',    '{"by": 3, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.269140900Z", "action": null, "subject": null}'),
                        ('f564b30a-98c4-494a-ad97-8e82ed27c0d8',    '{"by": 4, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.271736900Z", "action": null, "subject": null}'),
                        ('93835ce6-bace-4a6c-92ae-8f6829ee8ad0',    '{"by": 4, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.274350700Z", "action": null, "subject": null}'),
                        ('451e9e2e-b023-4e85-8376-27d20f4c3086',    '{"by": 1, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.277131100Z", "action": null, "subject": null}'),
                        ('73a413d3-4bcf-4fe6-a9c5-62da79cc223a',    '{"by": 2, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.279603300Z", "action": null, "subject": null}');
                    "#).unwrap();

                    FutOk((store, conn))
                })
                .and_then(move |(store, conn)| {
                    store
                        .aggregate("pg_store_pg_cache".into())
                        .map(|_entity: TestCounterEntity| (store, conn))
                })
                .and_then(move |(store, conn)| {
                    store
                        .aggregate("pg_store_pg_cache".into())
                        .map(|_entity: TestCounterEntity| (store, conn))
                })
                .and_then(|(store, conn)| {
                    conn.get().unwrap().batch_execute(r#"
                        INSERT INTO "events" ("id", "data", "context") VALUES
                        ('ea2391d7-1bf5-4375-b035-8e83e1299942',    '{"by": 1, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.132381900Z", "action": null, "subject": null}'),
                        ('12bccaed-9ce1-4865-850d-4c0fe2c2d0e8',    '{"by": 1, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.142785400Z", "action": null, "subject": null}'),
                        ('6dfe5d30-1b0a-4b90-a254-e43353ba19d2',    '{"by": 2, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.146425800Z", "action": null, "subject": null}'),
                        ('5c982759-40a3-4ca3-b93e-343e655ded17',    '{"by": 2, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.149805400Z", "action": null, "subject": null}'),
                        ('7de83490-7c97-478f-b4c7-0b2470274cc5',    '{"by": 3, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.153155800Z", "action": null, "subject": null}'),
                        ('e68f993e-6670-44ac-a081-786ded94fe07',    '{"by": 3, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.156322700Z", "action": null, "subject": null}'),
                        ('80953e27-627d-4235-98f2-47a789b3f30e',    '{"by": 4, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.159567Z", "action": null, "subject": null}'),
                        ('cff22bc7-67ba-408a-82ca-dfef3879d3ba',    '{"by": 4, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.162995Z", "action": null, "subject": null}'),
                        ('006a8207-2045-4b4f-8aab-591531810112',    '{"by": 1, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.166274900Z", "action": null, "subject": null}'),
                        ('b4939ab0-37f8-4b08-9565-e4ff828d0a88',    '{"by": 2, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.169911100Z", "action": null, "subject": null}');
                    "#).unwrap();

                    FutOk(store)
                })
                .and_then(move |store| store.aggregate("pg_store_pg_cache".into()))
                .and_then(|_: TestCounterEntity| FutOk(()));

                Runtime::new().unwrap().block_on_all(fut).unwrap();
            },
        );
    });
}

fn pg_store_redis_cache(c: &mut Criterion) {
    let conn = pg_create_random_db("exercise-redis");
    let redis_conn = redis_connect();

    c.bench_function("exercise pg store and redis cache", move |b| {
        b.iter_with_setup(
            || {
                pg_empty_events!(conn);
                redis_empty_cache!(redis_conn);
            },
            |_| {
                let _conn = conn.clone();
                let _redis = redis_conn.clone();

                let fut = lazy(move || {
                    let store = pg_store_with_redis_cache!(_conn, _redis);

                    FutOk((store, _conn))
                })
                .and_then(move |(store, conn)| {
                    conn.get().unwrap().batch_execute(r#"
                        INSERT INTO "events" ("id", "data", "context") VALUES
                        ('00dda037-21c8-41c4-9b00-39d3ac771af3',    '{"by": 1, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.240194600Z", "action": null, "subject": null}'),
                        ('37c67766-3ed1-4f42-83cb-4105d64bd672',    '{"by": 1, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.257805700Z", "action": null, "subject": null}'),
                        ('a63725e2-fcd6-44f2-8cb7-d7bf22ecf03c',    '{"by": 2, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.260671600Z", "action": null, "subject": null}'),
                        ('9ce4b0cd-f415-4091-b298-99812917dea1',    '{"by": 2, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.263277800Z", "action": null, "subject": null}'),
                        ('a477403d-b8ac-41a4-b5db-e712ee0f0f8b',    '{"by": 3, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.265985Z", "action": null, "subject": null}'),
                        ('c5e6297f-24bf-4f98-a13e-56511cd73a15',    '{"by": 3, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.269140900Z", "action": null, "subject": null}'),
                        ('f564b30a-98c4-494a-ad97-8e82ed27c0d8',    '{"by": 4, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.271736900Z", "action": null, "subject": null}'),
                        ('93835ce6-bace-4a6c-92ae-8f6829ee8ad0',    '{"by": 4, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.274350700Z", "action": null, "subject": null}'),
                        ('451e9e2e-b023-4e85-8376-27d20f4c3086',    '{"by": 1, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.277131100Z", "action": null, "subject": null}'),
                        ('73a413d3-4bcf-4fe6-a9c5-62da79cc223a',    '{"by": 2, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:29:35.279603300Z", "action": null, "subject": null}');
                    "#).unwrap();

                    FutOk((store, conn))
                })
                .and_then(move |(store, conn)| {
                    store
                        .aggregate("pg_store_pg_cache".into())
                        .map(|_entity: TestCounterEntity| (store, conn))
                })
                .and_then(move |(store, conn)| {
                    store
                        .aggregate("pg_store_pg_cache".into())
                        .map(|_entity: TestCounterEntity| (store, conn))
                })
                .and_then(|(store, conn)| {
                    conn.get().unwrap().batch_execute(r#"
                        INSERT INTO "events" ("id", "data", "context") VALUES
                        ('ea2391d7-1bf5-4375-b035-8e83e1299942',    '{"by": 1, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.132381900Z", "action": null, "subject": null}'),
                        ('12bccaed-9ce1-4865-850d-4c0fe2c2d0e8',    '{"by": 1, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.142785400Z", "action": null, "subject": null}'),
                        ('6dfe5d30-1b0a-4b90-a254-e43353ba19d2',    '{"by": 2, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.146425800Z", "action": null, "subject": null}'),
                        ('5c982759-40a3-4ca3-b93e-343e655ded17',    '{"by": 2, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.149805400Z", "action": null, "subject": null}'),
                        ('7de83490-7c97-478f-b4c7-0b2470274cc5',    '{"by": 3, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.153155800Z", "action": null, "subject": null}'),
                        ('e68f993e-6670-44ac-a081-786ded94fe07',    '{"by": 3, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.156322700Z", "action": null, "subject": null}'),
                        ('80953e27-627d-4235-98f2-47a789b3f30e',    '{"by": 4, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.159567Z", "action": null, "subject": null}'),
                        ('cff22bc7-67ba-408a-82ca-dfef3879d3ba',    '{"by": 4, "type": "some_namespace.TestDecrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestDecrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.162995Z", "action": null, "subject": null}'),
                        ('006a8207-2045-4b4f-8aab-591531810112',    '{"by": 1, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.166274900Z", "action": null, "subject": null}'),
                        ('b4939ab0-37f8-4b08-9565-e4ff828d0a88',    '{"by": 2, "type": "some_namespace.TestIncrementEvent", "ident": "pg_store_pg_cache", "event_type": "TestIncrementEvent", "event_namespace": "some_namespace"}', '{"time": "2018-11-03T12:31:11.169911100Z", "action": null, "subject": null}');
                    "#).unwrap();

                    FutOk(store)
                })
                .and_then(move |store| store.aggregate("pg_store_pg_cache".into()))
                .and_then(|_: TestCounterEntity| FutOk(()));

                Runtime::new().unwrap().block_on_all(fut).unwrap();
            },
        );
    });
}

criterion_group!(
    name = postgres;

    config = Criterion::default()
        .warm_up_time(Duration::from_millis(1000))
        .sample_size(20)
        .measurement_time(Duration::from_millis(4000));

    targets =
        pg_store_pg_cache,
        pg_store_redis_cache
);
criterion_main!(postgres);
