extern crate event_store;

use event_store::adapters::{AMQPEmitterAdapter, EmitterAdapter};
use event_store::testhelpers::TestEvents;
use std::net::SocketAddr;

#[test]
fn emitter_exists() {
    let addr: SocketAddr = "127.0.0.1:5672".parse().unwrap();
    let mut adapter = AMQPEmitterAdapter::new(addr, "iris".into(), "testing_namespace".into());
    adapter.subscribe("".into(), |_e: &TestEvents| {});
    adapter.unsubscribe("".into());
    assert!(true);
}
