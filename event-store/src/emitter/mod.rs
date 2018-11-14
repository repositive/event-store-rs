use crate::event::Event;
use event_store_derive_internals::EventData;
use std::thread::JoinHandle;

pub mod amqp;

/// Listen for events
pub trait EmitterReceiver<ED>
where
    ED: EventData,
{
    fn subscribe<H>(&self, handler: H) -> JoinHandle<()>
    where
        H: Fn(Event<ED>) -> () + Send + 'static;
}

/// Emit events
pub trait EmitterSender<ED>
where
    ED: EventData,
{
    fn emit(&self, event: &Event<ED>);
}

pub trait EmitterAdapter<ED, TX, RX>
where
    ED: EventData,
    TX: EmitterSender<ED>,
    RX: EmitterReceiver<ED>,
{
    fn split(self) -> (TX, RX);
}
