#![feature(await_macro, async_await, futures_api)]
#![feature(pin)]
#![feature(arbitrary_self_types)]

use event_store::adapters::{AmqpEmitterAdapter, PgCacheAdapter, PgStoreAdapter, SubscribeOptions};
use event_store::Event;
use event_store::*;
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

            let pool = pg_create_random_db();
            let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();

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

            await!(store.subscribe::<TestEvent>(SubscribeOptions::default()))?;

            await!(store.save(&test_event))?;

            Ok(())
        },
    )
    // Required so Rust can figure out what type `E` is
    .map_err(|e: io::Error| e);

    Runtime::new().unwrap().block_on(fut).unwrap();
}
