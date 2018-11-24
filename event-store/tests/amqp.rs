extern crate env_logger;
extern crate event_store;
extern crate futures;
#[macro_use]
extern crate log;
extern crate serde_json;
extern crate tokio;

use event_store::adapters::{AMQPEmitterAdapter, EmitterAdapter};
use event_store::testhelpers::TestIncrementEvent;
use event_store::Event;
use futures::Future;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;

#[test]
/// The intention of this test is to send an event through AMQP using the adapter and validate
/// that a handler subscribed to that same event receives it.
fn amqp_emitter_emits_and_subscribes() {
    env_logger::init();

    let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();
    let (tx, rx) = mpsc::channel();
    let original_sh = Arc::new(Mutex::new(tx));
    let sh = original_sh.clone();

    let mut rt = Runtime::new().unwrap();

    let amqp = AMQPEmitterAdapter::new(addr, "iris".into());

    let fut = amqp
        .subscribe(move |_event: Event<TestIncrementEvent>| {
            trace!("Received test event");

            &sh.lock().unwrap().send(()).unwrap();
        })
        .and_then(move |_| {
            amqp.emit(&Event::from_data(TestIncrementEvent {
                by: 1,
                ident: "some_ident".into(),
            }))
        });

    rt.block_on(fut).unwrap();

    assert!(rx.recv_timeout(Duration::from_secs(5)).is_ok());
}
