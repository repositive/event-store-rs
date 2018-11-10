use std::thread::{self, JoinHandle};

type Event = u32;

#[derive(Debug)]
struct AMQPEmitterAdapter {
    sender: AMQPSender,
    receiver: AMQPReceiver,
}

impl AMQPEmitterAdapter {
    pub fn new() -> Self {
        Self {
            sender: AMQPSender::new(),
            receiver: AMQPReceiver {},
        }
    }

    pub fn split(self) -> (AMQPSender, AMQPReceiver) {
        (self.sender, self.receiver)
    }

    // pub fn sender_cloned(&self) -> AMQPSender {
    //     self.sender.clone()
    // }
}

#[derive(Debug, Clone)]
// TODO: Custom impl clone to open new TCP connection
struct AMQPSender {}

impl AMQPSender {
    pub fn new() -> Self {
        // TODO: Open TCP connection, keep handle on self. Connection cannot be cloned. Custom clone
        // impl must open a new connection.

        Self {}
    }
}

#[derive(Debug)]
struct AMQPReceiver {}

impl AMQPReceiver {
    pub fn subscribe<H>(&self, store: Store, handler: H) -> JoinHandle<()>
    where
        H: Fn(Event, &Store) -> () + Send + 'static,
    {
        thread::spawn(move || {
            println!("Subscribe");

            // TODO: Open Rabbit connection

            // TODO: Subscribe to Rabbit queue

            store.some_func();

            handler(123, &store);
        })
    }
}

#[derive(Clone, Debug)]
struct Store {
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
struct SubscribableStore {
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
