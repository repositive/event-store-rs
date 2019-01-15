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
fn emit_and_receive() {
    pretty_env_logger::init();

    let fut = backward(
        async {
            let test_event = Event::from_data(TestEvent { num: 100 });

            info!("Save and emit test");

            let sender_pool = pg_create_random_db(Some("sender"));
            let receiver_pool = pg_create_random_db(Some("receiver"));
            let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();

            let sender_store = await!(SubscribableStore::new(
                await!(PgStoreAdapter::new(sender_pool.clone()))?,
                await!(PgCacheAdapter::new(sender_pool.clone()))?,
                await!(AmqpEmitterAdapter::new(
                    addr,
                    "test_exchange".into(),
                    "save_and_aggregate_send".into()
                ))?
            ))?;

            let receiver_store = await!(SubscribableStore::new(
                await!(PgStoreAdapter::new(receiver_pool.clone()))?,
                await!(PgCacheAdapter::new(receiver_pool.clone()))?,
                await!(AmqpEmitterAdapter::new(
                    addr,
                    "test_exchange".into(),
                    "save_and_aggregate_receive".into()
                ))?
            ))?;

            await!(receiver_store.subscribe::<TestEvent>(SubscribeOptions::default()))?;

            // Give time for subscriber to settle
            await!(forward(Delay::new(
                Instant::now() + Duration::from_millis(100)
            )))
            .unwrap();

            await!(sender_store.save(&test_event))?;

            // Wait for event to be received and stored before freeing everything
            await!(forward(Delay::new(
                Instant::now() + Duration::from_millis(100)
            )))
            .unwrap();

            let arg = &String::new();
            let result: TestCounterEntity = await!(receiver_store.aggregate(arg))?;

            assert_eq!(result, TestCounterEntity { counter: 100 });

            Ok(())
        },
    )
    // Required so Rust can figure out what type `E` is
    .map_err(|e: io::Error| e);

    Runtime::new().unwrap().block_on(fut).unwrap();
}
