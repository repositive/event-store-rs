#[macro_use]
extern crate log;
extern crate event_store;
extern crate pretty_env_logger;

use event_store::*;
use futures::future;
use futures::prelude::*;
use tokio_core::reactor::Core;

#[test]
fn save_and_aggregate() {
    pretty_env_logger::init();

    let test_event = TestEvent { num: 100 };
    let test_event_2 = TestEvent { num: 200 };

    trace!("Save and emit test");

    let pool = pg_create_random_db();

    let conn = pool.get().unwrap();

    let event_saver = EventSaver::new(pool.clone());

    let mut core = Core::new().unwrap();

    let run = event_saver
        .save(&Event::from_data(test_event))
        .join(event_saver.save(&Event::from_data(test_event_2)))
        .and_then(|_| pg_read(conn, TestCounterEntity::query(String::new()), None))
        .and_then(|events: Vec<TestEvents>| {
            future::ok(
                events
                    .iter()
                    .fold(TestCounterEntity::default(), TestCounterEntity::apply_event),
            )
        })
        .and_then(|aggregate| {
            info!("Aggregate result {:?}", aggregate);

            future::ok(())
        })
        .map_err(|e| {
            error!("Run error: {}", e);

            ()
        });

    core.run(run).unwrap();
}
