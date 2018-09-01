//! Stub emitter implementation

use adapters::EmitterAdapter;
use futures::future::{ok, Future};
use std::io::Error;
use Event;
use EventData;
use Events;

/// Stub event emitter
pub struct StubEmitterAdapter {}

impl StubEmitterAdapter {
    /// Create a new emitter stub
    pub fn new() -> Self {
        Self {}
    }
}

impl EmitterAdapter for StubEmitterAdapter {
    fn emit<E: Events>(
        &self,
        _event: &Event<E>,
    ) -> Box<Future<Item = (), Error = Error> + Send + Sync> {
        Box::new(ok(()))
    }

    fn subscribe<ED: EventData, H>(
        &self,
        _handler: H,
    ) -> Box<Future<Item = (), Error = Error> + Send>
    where
        H: Fn(&Event<ED>) -> (),
    {
        Box::new(ok(()))
    }
}
