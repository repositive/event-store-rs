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
use crate::store::StoreAdapter;
use crate::store::StoreQuery;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use futures::future::{ok as FutOk, Future};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use std::fmt::Debug;
use std::net::SocketAddr;
use std::thread::JoinHandle;
use tokio::runtime::Runtime;

trait EventStore<E, Q, S>
where
    E: Events + Debug,
    Q: StoreQuery,
    S: StoreAdapter<E, Q> + Clone + Send + 'static,
{
    fn save<ED>(&self, event: &Event<ED>) -> Result<(), String>
    where
        ED: EventData + Debug;
}

trait SubscribableEventStore<E, Q, S>: EventStore<E, Q, S>
where
    E: Events + Debug,
    Q: StoreQuery,
    S: StoreAdapter<E, Q> + Clone + Send + 'static,
{
    fn subscribe<H, ED>(&self, handler: H) -> JoinHandle<()>
    where
        ED: EventData + Debug,
        H: Fn(Event<ED>, &Store<S>) -> () + Send + Sync + 'static;
}

#[derive(Clone, Debug)]
pub struct Store<S> {
    emitter: AMQPSender,
    cache: PgCacheAdapter,
    store: S,
}

impl<S> Store<S> {
    pub fn new(emitter: AMQPSender, cache: PgCacheAdapter, store: S) -> Self {
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
}

impl<E, Q, S> EventStore<E, Q, S> for Store<S>
where
    E: Events + Debug,
    Q: StoreQuery,
    S: StoreAdapter<E, Q> + Clone + Send + 'static,
{
    fn save<ED>(&self, event: &Event<ED>) -> Result<(), String>
    where
        ED: EventData + Debug,
    {
        trace!("Store save");

        self.store.save(event).map(|_| {
            trace!("Save no emit complete");

            self.emitter.emit(event);
        })
    }

    // fn save_no_emit(&self, event: &Event<TestEvent>) -> Result<(), String> {
    //     self.store.save(event)
    // }
}

#[derive(Debug)]
pub struct SubscribableStore<S> {
    // Only this is clonable
    _store: Store<S>,

    // emitter: AMQPEmitterAdapter,
    receiver: AMQPReceiver,
}

impl<S> SubscribableStore<S> {
    pub fn new(emitter: AMQPEmitterAdapter, cache: PgCacheAdapter, store: S) -> Self {
        let (sender, receiver) = emitter.split();

        Self {
            _store: Store::new(sender, cache, store),
            receiver,
        }
    }
}

impl<E, Q, S> EventStore<E, Q, S> for SubscribableStore<S>
where
    E: Events + Debug,
    Q: StoreQuery,
    S: StoreAdapter<E, Q> + Clone + Send + 'static,
{
    fn save<ED>(&self, event: &Event<ED>) -> Result<(), String>
    where
        ED: EventData + Debug,
    {
        self._store.save(event)
    }
}

impl<E, Q, S> SubscribableEventStore<E, Q, S> for SubscribableStore<S>
where
    E: Events + Debug,
    Q: StoreQuery,
    S: StoreAdapter<E, Q> + Clone + Send + 'static,
{
    fn subscribe<H, ED>(&self, handler: H) -> JoinHandle<()>
    where
        ED: EventData + Debug,
        H: Fn(Event<ED>, &Store<S>) -> () + Send + Sync + 'static,
    {
        trace!("Store subscribe called");

        let handler_store = self._store.clone();

        self.receiver.subscribe(move |event: Event<ED>| {
            trace!("Received event {}", event.id);

            // TODO: How should store save errors be handled in event handlers?
            let _ = handler_store
                .save(&event)
                .map(|_| {
                    handler(event, &handler_store);

                    trace!("Handler called for event {}", event.id);
                })
                .map_err(|e| {
                    debug!("Handler not called: {}", e);
                });
        })
    }
}

#[derive(EventData, Debug)]
#[event_store(namespace = "some_namespace")]
struct TestEvent {
    num: u32,
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

    // let cache = PgCacheAdapter::new(pool.clone(), &set_stmt, &get_stmt);
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

        let test_fut: Box<Future<Item = u32, Error = ()> + Send> = Box::new(FutOk(123));

        let test_res = Runtime::new().unwrap().block_on_all(test_fut).unwrap();

        debug!("TEST RES {}", test_res);

        st.some_other_func();
    });

    trace!("SUBBED");

    store
        .save(&Event::from_data(TestEvent { num: 10 }))
        .expect("Could not save");

    trace!("--- END ---");

    loop {}
}
