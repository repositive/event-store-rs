//! Stub emitter implementation

use super::EmitterAdapter;
use event::Event;
use event_store_derive_internals::EventData;
use futures::future::ok as FutOk;
use futures::Future;
use serde_json::Value as JsonValue;
use std::io::Error;
use std::thread::{self, JoinHandle};
use utils::BoxedFuture;

/// Stub event emitter
#[derive(Clone)]
pub struct StubEmitterAdapter {}

impl StubEmitterAdapter {
    /// Create a new emitter stub
    pub fn new() -> Self {
        Self {}
    }
}

impl EmitterAdapter for StubEmitterAdapter {
    fn emit<E: EventData>(&self, _event: &Event<E>) -> BoxedFuture<(), ()> {
        Box::new(FutOk(()))
    }

    fn emit_with_string_ident(
        &self,
        _event_namespace: &str,
        _event_type: &str,
        _event: &JsonValue,
    ) -> BoxedFuture<(), ()> {
        Box::new(FutOk(()))
    }

    fn subscribe<ED, H>(&self, _handler: H) -> BoxedFuture<(), ()>
    where
        ED: EventData + 'static,
        H: Fn(Event<ED>) -> (),
    {
        // thread::spawn(move || {
        //     println!("Stub subscribe");
        // })

        Box::new(FutOk(()))
    }
}
