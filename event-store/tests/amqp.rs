extern crate event_store;
extern crate futures;
extern crate serde_json;
extern crate tokio;

#[macro_use]
extern crate log;
extern crate env_logger;

use event_store::adapters::{AMQPEmitterAdapter, EmitterAdapter};
use event_store::testhelpers::{TestEvents, TestIncrementEvent};
use event_store::Event;
use futures::future::Future;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

#[test]
/// The intention of this test is to send an event through AMQP using the adapter and validate
/// that a handler subscribed to that same event receives it.
fn emiter_emits_and_subscribes() {
    env_logger::init();
    let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();
    let mut runtime = Runtime::new().expect("Create runtime");

    // The adapter is configured in such a way that it can't receive messages that it sent.
    // For this reason we need an adapter to emit and another to subscribe
    let mut subscribe_adapter = AMQPEmitterAdapter::new(
        addr,
        "iris".into(),
        "testing_namespace".into(),
        &mut runtime,
    );

    let emit_adapter = AMQPEmitterAdapter::new(
        addr,
        "iris".into(),
        "testing_namespace".into(),
        &mut runtime,
    );

    let (tx, rx) = mpsc::channel();
    let original_sh = Arc::new(Mutex::new(tx));

    let sh = original_sh.clone();
    let subscription = subscribe_adapter.subscribe(move |_e: &Event<TestIncrementEvent>| {
        &sh.lock().unwrap().send(()).unwrap();
    });

    runtime.spawn(subscription.map_err(|e| error!("Something went south: {}", e)));

    // Wait for the handler to be ready before emit with an optimistic sync
    thread::sleep(Duration::from_millis(100));

    runtime.spawn(
        emit_adapter
            .emit(&Event::from_data(TestEvents::Inc(TestIncrementEvent {
                by: 1,
                ident: "".into(),
            }))).map_err(|_| ()),
    );

    assert!(rx.recv_timeout(Duration::from_secs(5)).is_ok());
}
