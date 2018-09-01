extern crate event_store;
extern crate futures;
extern crate serde_json;
extern crate tokio;

#[macro_use]
extern crate log;
extern crate env_logger;

use event_store::adapters::{AMQPEmitterAdapter, EmitterAdapter};
use event_store::testhelpers::{TestEvents, TestIncrementEvent};
use event_store::{Event, Events};
use futures::future::{ok, Future};
use futures::IntoFuture;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

#[test]
fn emiter_emits_and_subscribes() {
    env_logger::init();
    let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();
    let mut runtime = Runtime::new().expect("Create runtime");
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

    // Wait for the handler to be ready before emit. With an optimistic sync
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
