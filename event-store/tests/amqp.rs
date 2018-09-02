extern crate event_store;
extern crate futures;
extern crate serde_json;
extern crate tokio;

extern crate env_logger;

use event_store::adapters::{AMQPEmitterAdapter, EmitterAdapter};
use event_store::testhelpers::{TestEvents, TestIncrementEvent};
use event_store::Event;
use futures::future::Future;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;

#[test]
/// The intention of this test is to send an event through AMQP using the adapter and validate
/// that a handler subscribed to that same event receives it.
fn emitter_emits_and_subscribes() {
    env_logger::init();
    let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();
    let mut runtime = Runtime::new().expect("Create runtime");
    let (tx, rx) = mpsc::channel();
    let original_sh = Arc::new(Mutex::new(tx));
    let sh = original_sh.clone();

    // The adapter is configured in such a way that it can't receive messages that it sent.
    // For this reason we need an adapter to emit and another to subscribe
    let task = AMQPEmitterAdapter::new(addr, "iris".into())
        .and_then(move |adapter| {
            adapter.subscribe(move |_e: &Event<TestIncrementEvent>| {
                // Message received, let the main thread know about it.
                &sh.lock().unwrap().send(()).unwrap();
            })
        }).and_then(move |_| AMQPEmitterAdapter::new(addr, "iris".into()))
        .and_then(|adapter| {
            adapter.emit(&Event::from_data(TestEvents::Inc(TestIncrementEvent {
                by: 1,
                ident: "".into(),
            })))
        }).map_err(|_| ());

    runtime.spawn(task);

    assert!(rx.recv_timeout(Duration::from_secs(5)).is_ok());
}
