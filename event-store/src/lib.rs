use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

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

    pub fn emitter_cloned(&self) -> EventSender {
        self.tx.clone()
    }

    // pub fn split(self) -> (EventSender, EventReceiver) {
    //     (self.tx, self.rx)
    // }

    pub fn subscribe<H>(store: Store, handler: H) -> JoinHandle<()>
    where
        H: Fn(u32, &Store) -> () + Send + 'static,
    {
        thread::spawn(move || {
            println!("Subscribe");

            store.some_func();

            handler(123, &store);
        })
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

    pub fn some_func(&self) {
        println!("Call store func");
    }

    pub fn some_other_func(&self) {
        println!("Store func in handler");
    }
}

#[derive(Debug)]
struct SubscribableStore {
    // Only this is clonable
    _store: Store,

    emitter: AMQPEmitterAdapter,
}

impl SubscribableStore {
    pub fn new(emitter: AMQPEmitterAdapter) -> Self {
        // let (emitter, receiver) = emitter.split();
        let _emitter = emitter.emitter_cloned();

        Self {
            _store: Store::new(_emitter),
            emitter,
        }
    }

    pub fn subscribe<H>(&self, handler: H)
    where
        H: Fn(u32, &Store) -> () + Send + 'static,
    {
        let handler_store = self._store.clone();

        let _handle = AMQPEmitterAdapter::subscribe(handler_store, handler);
    }
}

#[test]
fn it_works() {
    let emitter = AMQPEmitterAdapter::new();
    let store = SubscribableStore::new(emitter);

    store.subscribe(|num, st| {
        println!("I'm in a handler. Num: {}", num);

        st.some_other_func();
    })
}
