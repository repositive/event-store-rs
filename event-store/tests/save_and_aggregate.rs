#[macro_use]
extern crate log;
extern crate event_store;
extern crate pretty_env_logger;

use event_store::*;
use futures::future;
use futures::prelude::*;
use std::io;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::timer::Delay;
use tokio_core::reactor::Core;

#[test]
fn save_and_aggregate() {
    pretty_env_logger::init();

    let test_event = TestEvent { num: 100 };
    let test_event_2 = TestEvent { num: 200 };

    trace!("Save and aggregate test");

    let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();
    let pool = pg_create_random_db();

    let conn = pool.get().unwrap();

    let event_saver = EventSaver::new(pool.clone());

    let mut core = Core::new().unwrap();

    let when = Instant::now() + Duration::from_millis(100);

    let run = amqp_connect(addr, "test_exchange".into())
        .and_then(move |channel| {
            store_save(
                event_saver.clone(),
                channel.clone(),
                Event::from_data(test_event),
            )
            .join(
                Delay::new(when)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                    .and_then(move |_| {
                        store_save(
                            event_saver.clone(),
                            channel.clone(),
                            Event::from_data(test_event_2),
                        )
                    }),
            )
        })
        .and_then(|_| store_aggregate(conn, String::new()))
        .and_then(|aggregate: TestCounterEntity| {
            info!("Aggregate result {:?}", aggregate);

            future::ok(())
        })
        .map_err(|e| {
            error!("Run error: {}", e);

            ()
        });

    core.run(run).unwrap();
}
