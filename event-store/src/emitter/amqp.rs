use crate::{Event, Store};
use std::thread::{self, JoinHandle};

#[derive(Debug)]
pub struct AMQPEmitterAdapter {
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
pub struct AMQPSender {}

impl AMQPSender {
    pub fn new() -> Self {
        // TODO: Open TCP connection, keep handle on self. Connection cannot be cloned. Custom clone
        // impl must open a new connection.

        Self {}
    }
}

#[derive(Debug)]
pub struct AMQPReceiver {}

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
