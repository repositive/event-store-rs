use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Clone, Debug)]
struct EventSender {
    tx: Sender<u32>,
}

impl EventSender {
    pub fn new(tx: Sender<u32>) -> Self {
        Self { tx }
    }
}

#[derive(Debug)]
struct EventReceiver {
    rx: Receiver<u32>,
}

impl EventReceiver {
    pub fn new(rx: Receiver<u32>) -> Self {
        Self { rx }
    }
}

#[derive(Debug)]
struct AMQPEmitterAdapter {
    tx: EventSender,
    rx: EventReceiver,
}

impl AMQPEmitterAdapter {
    pub fn new() -> Self {
        let (tx, rx) = channel();

        Self {
            tx: EventSender::new(tx),
            rx: EventReceiver::new(rx),
        }
    }

    pub fn split(self) -> (EventSender, EventReceiver) {
        (self.tx, self.rx)
    }
}

#[derive(Clone, Debug)]
struct Store {
    emitter: EventSender,
}

impl Store {
    pub fn new(emitter: EventSender) -> Self {
        Self { emitter }
    }
}

#[derive(Debug)]
struct SubscribableStore {
    // Only this is clonable
    _store: Store,

    // This is not clonable because it keeps a receiver which can only be owned by one thread
    receiver: EventReceiver,
}

impl SubscribableStore {
    pub fn new(emitter: AMQPEmitterAdapter) -> Self {
        let (emitter, receiver) = emitter.split();

        Self {
            _store: Store::new(emitter),
            receiver,
        }
    }

    pub fn subscribe<H>(&self, handler: H)
    where
        H: Fn(u32) -> (),
    {
        let receiver_store = self._store.clone();
    }
}
