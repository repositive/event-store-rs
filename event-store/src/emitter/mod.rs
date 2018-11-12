pub mod amqp;

/// Listen for events
pub trait EmitterReceiver {}

/// Emit events
pub trait EmitterSender {}

pub trait EmitterAdapter {}
