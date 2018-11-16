use event::Event;
use event_store_derive_internals::EventData;
use std::io;
use std::thread::JoinHandle;

mod amqp;
mod stub;

pub use self::amqp::AMQPEmitterAdapter;
pub use self::stub::StubEmitterAdapter;

/// Event emitter interface
pub trait EmitterAdapter: Clone + Send + 'static {
    /// Emit an event
    fn emit<E: EventData + Send>(&self, event: &Event<E>) -> Result<(), io::Error>;

    /// Subscribe to an event
    fn subscribe<ED, H>(&self, handler: H) -> JoinHandle<()>
    where
        ED: EventData + 'static,
        H: Fn(Event<ED>) -> () + Send + 'static;
}
