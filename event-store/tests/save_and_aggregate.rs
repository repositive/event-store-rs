#![feature(await_macro, async_await, futures_api)]
#![feature(arbitrary_self_types)]

use event_store::adapters::{AmqpEmitterAdapter, PgCacheAdapter, PgStoreAdapter};
use event_store::internals::{backward, forward, test_helpers::*};
use event_store::prelude::*;
use event_store::SubscribableStore;
use futures::future::Future;
use log::trace;
use std::io;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::timer::Delay;

#[test]
fn save_and_aggregate() {
    pretty_env_logger::init();

    let fut = backward(
        async {
            let test_event = Event::from_data(TestEvent { num: 100 });
            let test_event_2 = Event::from_data(TestEvent { num: 200 });

            trace!("Save and aggregate test");

            let pool = pg_create_random_db(None);
            let addr = "amqp://localhost:5673";

            let store_adapter = await!(PgStoreAdapter::new(pool.clone()))?;
            let cache_adapter = await!(PgCacheAdapter::new(pool.clone()))?;
            let emitter_adapter = await!(AmqpEmitterAdapter::new(
                addr,
                "test_exchange".into(),
                "save_and_aggregate".into()
            ))?;

            let store = await!(SubscribableStore::new(
                store_adapter,
                cache_adapter,
                emitter_adapter
            ))?;

            await!(forward(Delay::new(
                Instant::now() + Duration::from_millis(100)
            )))
            .unwrap();

            await!(store.save(&test_event))?;
            await!(store.save(&test_event_2))?;

            let arg = &String::new();

            let uncached_result: TestCounterEntity = await!(store.aggregate(arg))?;

            let cached_result: TestCounterEntity = await!(store.aggregate(arg))?;

            Ok((uncached_result, cached_result))
        },
    )
    // Required so Rust can figure out what type `E` is
    .map_err(|e: io::Error| e);

    let (uncached_result, cached_result) = Runtime::new().unwrap().block_on(fut).unwrap();

    assert_eq!(uncached_result, TestCounterEntity { counter: 300i32 });
    assert_eq!(uncached_result, cached_result);
}
