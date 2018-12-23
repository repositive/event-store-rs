#![feature(await_macro, async_await, futures_api)]
#![feature(pin)]
#![feature(arbitrary_self_types)]

use event_store::*;
use futures::prelude::*;
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

            let pool = pg_create_random_db();

            let store = await!(SubscribableStore::new("store_namespace".into(), pool))?;

            await!(forward(Delay::new(
                Instant::now() + Duration::from_millis(100)
            )))
            .unwrap();

            await!(store.save(&test_event))?;
            await!(store.save(&test_event_2))?;

            let arg = &String::new();

            let result: TestCounterEntity = await!(store.aggregate(arg))?;

            Ok(result)
        },
    )
    // Required so Rust can figure out what type `E` is
    .map_err(|e: io::Error| e);

    let result = Runtime::new().unwrap().block_on(fut).unwrap();

    assert_eq!(result, TestCounterEntity { counter: 300i32 });
}
