#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

mod cache;
mod emitter;
mod store;

use crate::cache::pg::PgCacheAdapter;
use crate::emitter::amqp::{AMQPEmitterAdapter, AMQPReceiver, AMQPSender};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use std::net::SocketAddr;
use std::thread::JoinHandle;

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

    pub fn subscribe<H>(&self, handler: H) -> JoinHandle<()>
    where
        H: Fn(Event, &Store) -> () + Send + 'static,
    {
        trace!("Store subscribe called");

        let handler_store = self._store.clone();

        self.receiver.subscribe(handler_store, handler)
    }
}

#[test]
fn it_works() {
    pretty_env_logger::init();

    trace!("Start...");

    let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();

    let manager = PostgresConnectionManager::new(
        "postgres://postgres:postgres@localhost:5430/eventstorerust",
        TlsMode::None,
    )
    .unwrap();

    let pool = r2d2::Pool::new(manager).unwrap();

    let cache_conn = pool.get().unwrap();

    let (set_stmt, get_stmt) = PgCacheAdapter::prepare_statements(&cache_conn);

    let cache = PgCacheAdapter::new(&set_stmt, &get_stmt);

    let emitter = AMQPEmitterAdapter::new(addr, "iris".into());

    trace!("Emitter all done");

    let store = SubscribableStore::new(emitter);

    trace!("All done");

    let _handle = store.subscribe(|num, st| {
        println!("I'm in a handler. Num: {}", num);

        st.some_other_func();
    });

    trace!("SUBBED");

    loop {}
}
