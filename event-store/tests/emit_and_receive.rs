#![feature(await_macro, async_await, futures_api)]
#![feature(pin)]
#![feature(arbitrary_self_types)]

use event_store::Event;
use event_store::*;
use futures::future::Future;
use log::{debug, info};
use std::io;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::timer::Delay;

#[test]
fn save_and_emit() {
    pretty_env_logger::init();

    let fut = backward(
        async {
            let test_event = Event::from_data(TestEvent { num: 100 });

            info!("Save and emit test");

            let pool = pg_create_random_db();

            let store = await!(SubscribableStore::new("store_namespace".into(), pool)).unwrap();

            store.subscribe::<TestEvent>();

            await!(store.save(&test_event)).unwrap();

            Ok(())
        },
    )
    // Required so Rust can figure out what type `E` is
    .map_err(|e: io::Error| e);

    Runtime::new().unwrap().block_on(fut).unwrap();
}
