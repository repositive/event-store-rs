use event_store::*;
use futures::future;
use futures::prelude::*;
use log::{info, trace};
use std::io;
use std::time::{Duration, Instant};
use tokio::timer::Delay;
use tokio_core::reactor::Core;

#[test]
fn save_and_aggregate() {
    pretty_env_logger::init();

    let test_event = TestEvent { num: 100 };
    let test_event_2 = TestEvent { num: 200 };

    trace!("Save and aggregate test");

    let pool = pg_create_random_db();

    let mut core = Core::new().unwrap();

    let when = Instant::now() + Duration::from_millis(100);

    let run = Store::new("store_namespace".into(), pool)
        .and_then(|store| {
            let save_again = store.save(Event::from_data(test_event_2));

            store
                .save(Event::from_data(test_event))
                .join(
                    // FIXME: Due to internal issues with lapin_futures, it's not currently possible to
                    // save two events at the same time, hence the delay.
                    Delay::new(when)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                        .and_then(|_| save_again),
                )
                .map(|_| store)
        })
        .and_then(|store| store.aggregate(String::new()))
        .and_then(|aggregate: TestCounterEntity| {
            info!("Aggregate result {:?}", aggregate);

            future::ok(())
        });

    core.run(run).unwrap();
}
