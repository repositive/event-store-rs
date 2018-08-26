//! Stub emitter implementation

use adapters::{EmitterAdapter, EventHandler};
use std::collections::HashMap;
use Events;

/// Stub event emitter
pub struct StubEmitterAdapter<E> {
    subscribers: HashMap<String, EventHandler<E>>,
}

impl<E> StubEmitterAdapter<E> {
    /// Create a new emitter stub
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
    }
}

impl<E> EmitterAdapter<E> for StubEmitterAdapter<E>
where
    E: Events,
{
    fn get_subscriptions(&self) -> &HashMap<String, EventHandler<E>> {
        &self.subscribers
    }

    fn emit(&self, _event: &E) {}

    fn subscribe(&mut self, _event_name: String, _handler: EventHandler<E>) {}

    fn unsubscribe(&mut self, _event_name: String) {}
}
