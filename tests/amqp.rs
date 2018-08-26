extern crate event_store_rs;

use event_store_rs::adapters::{AMQPEmitterAdapter, EmitterAdapter};
use event_store_rs::testhelpers::{
    TestCounterEntity, TestDecrementEvent, TestEvents, TestIncrementEvent,
};

#[test]
fn emitter_exists() {
    let mut adapter = AMQPEmitterAdapter::new();
    adapter.subscribe("".into(), |_e: &TestEvents| {});
    adapter.unsubscribe("".into());
    assert!(true);
}
