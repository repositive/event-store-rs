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

    let test_event = TestEvent { num: 100 };

    info!("Save and emit test");

    let pool = pg_create_random_db();

    let mut rt = Runtime::new().unwrap();

    let run = SubscribableStore::new("store_namespace".into(), pool)
        .and_then(|store| {
            debug!("Store created");

            store.subscribe::<TestEvent>().map(|_| store)
        })
        .and_then(|store| {
            debug!("Subscription created");
            store.save(Event::from_data(test_event)).map(|_| store)
        })
        .and_then(|_| {
            debug!("Store saved");
            Delay::new(Instant::now() + Duration::from_millis(100))
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "wait error"))
        })
        .map(|_| debug!("Chain complete"));

    rt.block_on(run).unwrap();
}
