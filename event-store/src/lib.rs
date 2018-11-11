#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate event_store_derive;

mod cache;
mod emitter;
mod event;
mod event_context;
mod store;

use crate::cache::pg::PgCacheAdapter;
use crate::emitter::amqp::{AMQPEmitterAdapter, AMQPReceiver, AMQPSender};
use crate::event::Event;
use crate::store::pg::PgStoreAdapter;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use std::net::SocketAddr;
use std::thread::JoinHandle;

// type Event = u32;

/// Test event
#[derive(EventData, Debug)]
#[event_store(namespace = "some_namespace")]
pub struct TestEvent {
    pub num: u32,
}

#[derive(Events, Debug)]
pub enum TestEvents {
    Test(Event<TestEvent>),
}

#[derive(Clone, Debug)]
pub struct Store<'a> {
    emitter: AMQPSender,
    cache: PgCacheAdapter<'a>,
    store: PgStoreAdapter,
}

impl<'a> Store<'a> {
    pub fn new(emitter: AMQPSender, cache: PgCacheAdapter<'a>, store: PgStoreAdapter) -> Self {
        Self {
            emitter,
            cache,
            store,
        }
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
pub struct SubscribableStore<'a> {
    // Only this is clonable
    _store: Store<'a>,

    // emitter: AMQPEmitterAdapter,
    receiver: AMQPReceiver,
}

impl<'a> SubscribableStore<'a> {
    pub fn new(
        emitter: AMQPEmitterAdapter,
        cache: PgCacheAdapter<'a>,
        store: PgStoreAdapter,
    ) -> Self {
        let (sender, receiver) = emitter.split();

        Self {
            _store: Store::new(sender, cache, store),
            receiver,
        }
    }

    pub fn subscribe<H>(&self, handler: H) -> JoinHandle<()>
    where
        H: Fn(Event<TestEvent>, &Store) -> () + Send + 'static,
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
    let store = PgStoreAdapter::new(pool.clone());
    // let cache = PgCacheAdapter::new(pool.clone());

    let emitter = AMQPEmitterAdapter::new(addr, "iris".into());

    trace!("Emitter all done");

    let store = SubscribableStore::new(emitter, cache, store);

    trace!("All done");

    let _handle = store.subscribe(|evt, st| {
        println!("I'm in a handler. Num: {:?}", evt);

        st.some_other_func();
    });

    trace!("SUBBED");

    loop {}
}
