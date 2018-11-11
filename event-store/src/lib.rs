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
use tokio::runtime::Runtime;

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
pub struct Store {
    emitter: AMQPSender,
    cache: PgCacheAdapter,
    store: PgStoreAdapter,
}

impl Store {
    pub fn new(emitter: AMQPSender, cache: PgCacheAdapter, store: PgStoreAdapter) -> Self {
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

    pub fn save(&self, event: &Event<TestEvent>) -> Result<(), String> {
        trace!("Store save");

        self.save_no_emit(event).map(|_| {
            trace!("Save no emit complete");

            self.emitter.emit(event);
        })
    }

    pub(crate) fn save_no_emit(&self, event: &Event<TestEvent>) -> Result<(), String> {
        self.store.save(event)
    }
}

#[derive(Debug)]
pub struct SubscribableStore {
    // Only this is clonable
    _store: Store,

    // emitter: AMQPEmitterAdapter,
    receiver: AMQPReceiver,
}

impl SubscribableStore {
    pub fn new(emitter: AMQPEmitterAdapter, cache: PgCacheAdapter, store: PgStoreAdapter) -> Self {
        let (sender, receiver) = emitter.split();

        Self {
            _store: Store::new(sender, cache, store),
            receiver,
        }
    }

    pub fn save(&self, event: &Event<TestEvent>) -> Result<(), String> {
        self._store.save(event)
    }

    pub fn subscribe<H>(&self, handler: H) -> JoinHandle<()>
    where
        H: Fn(Event<TestEvent>, &Store) -> () + Send + Sync + 'static,
    {
        trace!("Store subscribe called");

        let handler_store = self._store.clone();

        self.receiver.subscribe(handler_store, move |event, store| {
            trace!("Handler called for event {}", event.id);

            // TODO: How should store save errors be handled in event handlers?
            let _ = store
                .save_no_emit(&event)
                .map(|_| {
                    trace!("New event saved");

                    handler(event, store);
                })
                .map_err(|e| {
                    debug!("Handler not called: {}", e);
                });
        })
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

    // let cache_conn = pool.get().unwrap();

    // let (set_stmt, get_stmt) = PgCacheAdapter::prepare_statements(&cache_conn);

    // let cache = PgCacheAdapter::new(&set_stmt, &get_stmt);
    let cache = PgCacheAdapter::new(pool.clone());
    let store_adapter = PgStoreAdapter::new(pool.clone());
    // let cache = PgCacheAdapter::new(pool.clone());

    // This must be its own variable so it lives until program end, otherwise there will be no
    // runtime present when an event is emitted. This causes a panic.
    let mut emitter_rt = Runtime::new().unwrap();

    let emitter = emitter_rt
        .block_on(AMQPEmitterAdapter::new(addr, "exchange_here".into()))
        .unwrap();
    // let emitter = AMQPEmitterAdapter::new(addr, "iris".into());

    trace!("Emitter all done");

    let store = SubscribableStore::new(emitter, cache, store_adapter);

    trace!("All done");

    let _handle = store.subscribe(|evt, st| {
        println!("I'm in a handler. Num: {:?}", evt);

        st.some_other_func();
    });

    trace!("SUBBED");

    store
        .save(&Event::from_data(TestEvent { num: 10 }))
        .expect("Could not save");

    trace!("--- END ---");

    loop {}
}
