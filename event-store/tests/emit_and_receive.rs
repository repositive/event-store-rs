#[macro_use]
extern crate log;
extern crate event_store;
extern crate pretty_env_logger;

use event_store::Event;
use event_store::*;
use futures::future::Future;
use std::io;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::timer::Delay;

#[test]
fn save_and_emit() {
    pretty_env_logger::init();

    let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();
    let test_event = TestEvent { num: 100 };

    trace!("Save and emit test");

    let conn = pg_create_random_db();

    let event_saver = EventSaver::new(conn.clone());

    let mut rt = Runtime::new().unwrap();

    let run = amqp_connect(addr, "test_exchange".into())
        .and_then(move |channel| {
            info!("AMQP connected");

            let consumer = amqp_create_consumer(
                channel.clone(),
                "rando_queue".into(),
                "test_exchange".into(),
                move |ev: Event<TestEvent>| {
                    debug!("Received event {}", ev.id);

                    event_saver.save(&ev);
                },
            );

            tokio::spawn(consumer.map_err(|e| {
                error!("Consumer error: {}", e);

                ()
            }));

            amqp_emit_event(
                channel.clone(),
                "rando_queue".into(),
                "test_exchange".into(),
                &Event::from_data(test_event),
            )
        })
        .and_then(|_| {
            Delay::new(Instant::now() + Duration::from_millis(100))
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "wait error"))
        })
        .map_err(|e| {
            error!("Run error: {}", e);

            ()
        });

    rt.block_on(run).unwrap();
}
