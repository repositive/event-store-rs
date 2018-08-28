//! Stub emitter implementation

use adapters::{EmitterAdapter, EventHandler};
use std::collections::HashMap;
use Event;
use EventData;

/// Stub event emitter
pub struct StubEmitterAdapter {}

impl StubEmitterAdapter {
    /// Create a new emitter stub
    pub fn new() -> Self {
        Self {}
    }
}

impl<E> EmitterAdapter<E> for StubEmitterAdapter
where
    E: EventData,
{
    fn get_subscriptions(&self) -> HashMap<String, EventHandler<E>> {
        HashMap::new()
    }

    fn emit(&self, _event: &Event<E>) {}

    fn subscribe<H>(&mut self, _event_name: String, _handler: EventHandler<E>) {}

    fn unsubscribe(&mut self, _event_name: String) {}
}
