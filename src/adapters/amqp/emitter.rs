//! AMQP emitter implementation

use adapters::{EmitterAdapter, EventHandler};
use std::collections::HashMap;
use std::marker::PhantomData;
use Events;

/// AMQP emitter
pub struct AMQPEmitterAdapter<E> {
    phantom: PhantomData<E>,
    subscribers: HashMap<String, EventHandler<E>>,
}

impl<E> AMQPEmitterAdapter<E> {
    /// Create a new AMQPEmiterAdapter
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            subscribers: HashMap::new(),
        }
    }
}

impl<E> EmitterAdapter<E> for AMQPEmitterAdapter<E>
where
    E: Events,
{
    fn get_subscriptions(&self) -> &HashMap<String, EventHandler<E>> {
        &self.subscribers
    }

    fn emit(&self, _event: &E) {}

    fn subscribe(&mut self, _event_name: String, _handler: EventHandler<E>) {
        &self.subscribers.insert(_event_name, _handler);
    }

    fn unsubscribe(&mut self, _event_name: String) {
        &self.subscribers.remove(&_event_name);
    }
}
