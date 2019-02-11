#![feature(await_macro, async_await, futures_api)]
#![feature(arbitrary_self_types)]

use event_store::adapters::{AmqpEmitterAdapter, PgCacheAdapter, PgStoreAdapter};
use event_store::internals::{backward, forward, test_helpers::*};
use event_store::prelude::*;
use event_store::SubscribableStore;
use futures::future::Future;
use log::info;
use std::io;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::timer::Delay;

#[test]
fn replay() {
    pretty_env_logger::init();

    let fut = backward(
        async {
            let test_event = Event::from_data(TestEvent { num: 100 });
            let test_event2 = Event::from_data(TestEvent { num: 200 });

            info!("Replay test");

            let pool = pg_create_random_db(Some("replay"));
            let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();

            let creator_store = await!(SubscribableStore::new(
                await!(PgStoreAdapter::new(
                    pool.clone(),
                    "replay-creator".into()
                ))?,
                await!(PgCacheAdapter::new(pool.clone()))?,
                await!(AmqpEmitterAdapter::new(
                    addr,
                    "test_exchange".into(),
                    "replay-create".into()
                ))?
            ))?;

            // Wait for store to settle
            await!(forward(Delay::new(
                Instant::now() + Duration::from_millis(200)
            )))
            .unwrap();

            await!(creator_store.save(&test_event))?;
            await!(creator_store.save(&test_event2))?;

            // Wait for event to be received and stored before freeing everything
            await!(forward(Delay::new(
                Instant::now() + Duration::from_millis(200)
            )))
            .unwrap();

            let consumer_store = await!(SubscribableStore::new(
                await!(PgStoreAdapter::new(
                    pool.clone(),
                    "replay-consumer".into()
                ))?,
                await!(PgCacheAdapter::new(pool.clone()))?,
                await!(AmqpEmitterAdapter::new(
                    addr,
                    "test_exchange".into(),
                    "replay-consume".into()
                ))?
            ))?;

            // Wait for stores to settle
            await!(forward(Delay::new(
                Instant::now() + Duration::from_millis(200)
            )))
            .unwrap();

            await!(consumer_store.subscribe::<TestEvent>())?;

            // Give time for subscriber to settle
            await!(forward(Delay::new(
                Instant::now() + Duration::from_millis(200)
            )))
            .unwrap();

            let arg = &String::new();
            let result: TestCounterEntity = await!(consumer_store.aggregate(arg))?;

            assert_eq!(result, TestCounterEntity { counter: 300 });

            Ok(())
        },
    )
    // Required so Rust can figure out what type `E` is
    .map_err(|e: io::Error| e);

    Runtime::new().unwrap().block_on(fut).unwrap();
}
