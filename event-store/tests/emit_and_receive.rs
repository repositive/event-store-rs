#![feature(await_macro, async_await)]
#![feature(arbitrary_self_types)]

use event_store::adapters::{AmqpEmitterAdapter, PgCacheAdapter, PgStoreAdapter};
use event_store::internals::{backward, forward, test_helpers::*};
use event_store::prelude::*;
use event_store::SubscribableStore;
use futures::future::Future;
use log::info;
use std::io;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::timer::Delay;

#[test]
fn emit_and_receive() {
    pretty_env_logger::init();

    let fut = backward(async {
        let test_event = Event::from_data(TestEvent { num: 100 });

        info!("Save and emit test");

        let pool = pg_create_random_db(Some("emit_and_receive"));
        let addr = "amqp://localhost:5673";

        let sender_store = SubscribableStore::new(
            await!(PgStoreAdapter::new(pool.clone()))?,
            await!(PgCacheAdapter::new(pool.clone()))?,
            await!(AmqpEmitterAdapter::new(
                addr,
                "test_exchange".into(),
                "save_and_aggregate_send".into()
            ))?,
        )?;

        let receiver_store = SubscribableStore::new(
            await!(PgStoreAdapter::new(pool.clone()))?,
            await!(PgCacheAdapter::new(pool.clone()))?,
            await!(AmqpEmitterAdapter::new(
                addr,
                "test_exchange".into(),
                "save_and_aggregate_receive".into()
            ))?,
        )?;

        await!(receiver_store.subscribe::<TestEvent>())?;

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
    })
    // Required so Rust can figure out what type `E` is
    .map_err(|e: io::Error| e);

    Runtime::new().unwrap().block_on(fut).unwrap();
}
