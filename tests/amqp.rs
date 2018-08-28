extern crate event_store;
extern crate serde_json;

use event_store::adapters::{AMQPEmitterAdapter, EmitterAdapter};
use event_store::testhelpers::{TestEvents, TestIncrementEvent};
use event_store::Event;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

#[test]
fn emiter_emits_and_subscribes() {
    let addr: SocketAddr = "127.0.0.1:5672".parse().unwrap();
    let mut adapter = AMQPEmitterAdapter::new(addr, "iris".into(), "testing_namespace".into());
    let (tx, rx) = mpsc::channel();
    let original_sh = Arc::new(Mutex::new(tx));

    let sh = original_sh.clone();
    adapter.subscribe(
        "some_namespace.Inc".into(),
        Box::new(move |_e: &Event<TestEvents>| {
            &sh.lock().unwrap().send(()).unwrap();
        }),
    );

    adapter.emit(&Event::from_data(TestEvents::Inc(TestIncrementEvent {
        by: 1,
        ident: "".into(),
    })));

    assert!(rx.recv_timeout(Duration::from_secs(1)).is_ok());
}
