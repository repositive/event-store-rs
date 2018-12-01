//! Stub emitter implementation

use super::EmitterAdapter;
use event::Event;
use event_store_derive_internals::EventData;
use futures::future::ok as FutOk;
use serde_json::Value as JsonValue;
use std::io;
use utils::BoxedFuture;

/// Stub event emitter
#[derive(Clone, Default)]
pub struct StubEmitterAdapter {}

impl StubEmitterAdapter {
    /// Create a new emitter stub
    pub fn new() -> Self {
        Self {}
    }
}

impl EmitterAdapter for StubEmitterAdapter {
    fn emit<ED: EventData>(&self, _event: &Event<ED>) -> BoxedFuture<(), io::Error> {
        Box::new(FutOk(()))
    }

    fn emit_with_string_ident(
        &self,
        _event_namespace: &str,
        _event_type: &str,
        _event: &JsonValue,
    ) -> BoxedFuture<(), io::Error> {
        Box::new(FutOk(()))
    }

    fn subscribe<ED, H>(&self, _handler: H) -> BoxedFuture<(), io::Error>
    where
        ED: EventData + 'static,
        H: Fn(Event<ED>) -> (),
    {
        Box::new(FutOk(()))
    }
}
