//! Stub emitter implementation

use adapters::{EmitterAdapter, EventHandler};
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
    fn emit(&mut self, _event: &Event<E>) {}

    fn subscribe(&mut self, _event_name: String, _: Box<EventHandler<E>>) {}
}
