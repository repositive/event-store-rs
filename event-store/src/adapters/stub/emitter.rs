//! Stub emitter implementation

use adapters::EmitterAdapter;
use event_store_derive_internals::EventData;
use futures::future::ok as FutOk;
use std::io::Error;
use utils::BoxedFuture;
use Event;

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
    fn emit<E: EventData>(&self, _event: &Event<E>) -> Result<(), Error> {
        Ok(())
    }

    fn subscribe<ED, H>(&self, _handler: H) -> BoxedFuture<(), ()>
    where
        ED: EventData + 'static,
        H: Fn(&Event<ED>) -> (),
    {
        Box::new(FutOk(()))
    }
}
