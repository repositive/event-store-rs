mod emitter;

use crate::emitter::amqp::{AMQPEmitterAdapter, AMQPReceiver, AMQPSender};

type Event = u32;

#[derive(Clone, Debug)]
pub struct Store {
    emitter: AMQPSender,
}

impl Store {
    pub fn new(emitter: AMQPSender) -> Self {
        Self { emitter }
    }

    pub fn some_func(&self) {
        println!("Call store func");
    }

    pub fn some_other_func(&self) {
        println!("Store func in handler");
    }

    // pub fn emit(&self) {
    //     // TODO
    // }
}

#[derive(Debug)]
pub struct SubscribableStore {
    // Only this is clonable
    _store: Store,

    // emitter: AMQPEmitterAdapter,
    receiver: AMQPReceiver,
}

impl SubscribableStore {
    pub fn new(emitter: AMQPEmitterAdapter) -> Self {
        let (sender, receiver) = emitter.split();

        Self {
            _store: Store::new(sender),
            receiver,
        }
    }

    pub fn subscribe<H>(&self, handler: H)
    where
        H: Fn(Event, &Store) -> () + Send + 'static,
    {
        let handler_store = self._store.clone();

        let _handle = self.receiver.subscribe(handler_store, handler);
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
