use super::adapters::{AMQPEmitterAdapter, PgCacheAdapter, PgStoreAdapter};
use super::testhelpers::TestIncrementEvent;
use super::utils::BoxedFuture;
use super::{Aggregator, Event, EventData, EventStore, Events, Store, StoreAdapter, StoreQuery};
use bb8::Pool;
use bb8_postgres::{PostgresConnectionManager, TlsMode};
use chrono::{DateTime, Utc};
use futures::future::ok as FutOk;
use std::net::SocketAddr;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;

enum MockedStoreChecks {
    Save(Box<Event<EventData> + Send>),
}

#[derive(Clone)]
struct MockedStoreAdapter {
    reporter: Arc<Mutex<Sender<MockedStoreChecks>>>,
}

impl MockedStoreAdapter {
    fn new(reporter: Arc<Mutex<Sender<MockedStoreChecks>>>) -> Self {
        Self { reporter }
    }
}

impl<Q: StoreQuery> StoreAdapter<Q> for MockedStoreAdapter {
    fn aggregate<E, T, A>(
        &self,
        query_Args: A,
        since: Option<(T, DateTime<Utc>)>,
    ) -> Result<T, String>
    where
        E: Events,
        T: Aggregator<E, A, Q> + Default,
        A: Clone,
        Q: StoreQuery,
    {
        &self
            .reporter
            .lock()
            .unwrap()
            .send("aggregate".into())
            .unwrap();
        Err("Not implemented!".into())
    }

    fn save<ED: EventData>(&self, event: &Event<ED>) -> Result<(), String> {
        &self.reporter.lock().unwrap().send("save".into()).unwrap();
        Ok(())
    }

    fn last_event<ED: EventData + Send + 'static>(&self) -> BoxedFuture<Option<Event<ED>>, String> {
        &self
            .reporter
            .lock()
            .unwrap()
            .send("last_event".into())
            .unwrap();
        Box::new(FutOk(None))
    }
}

#[test]
fn event_store_can_be_created() {
    let mut runtime = Runtime::new().unwrap();
    let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();
    let pg_manager = PostgresConnectionManager::new(
        "psql://postgres:eventstorerust@127.0.0.1:5430",
        TlsMode::None,
    ).unwrap();
    let connection = Pool::new(pg_manager).unwrap();
    let amqp = runtime
        .block_on(AMQPEmitterAdapter::new(addr, "iris".into()))
        .unwrap();
    let store = PgStoreAdapter::new(connection.clone());
    let cache = PgCacheAdapter::new(connection.clone());
    let _esstore = EventStore::new(store, cache, amqp);
}

#[test]
fn event_store_can_save_events() {
    let mut runtime = Runtime::new().unwrap();
    let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();
    let pg_manager = PostgresConnectionManager::new(
        "psql://postgres:eventstorerust@127.0.0.1:5430",
        TlsMode::None,
    ).unwrap();
    let connection = Pool::new(pg_manager).unwrap();
    let amqp = runtime
        .block_on(AMQPEmitterAdapter::new(addr, "iris".into()))
        .unwrap();
    let (tx, receiver) = channel();
    let sender = Arc::new(Mutex::new(tx));
    let store = MockedStoreAdapter::new(sender);
    let cache = PgCacheAdapter::new(connection.clone());
    let esstore = EventStore::new(store, cache, amqp);

    esstore.save(Event::from_data(TestIncrementEvent {
        by: 1,
        ident: "".into(),
    }));

    let result = receiver.recv_timeout(Duration::from_micros(1)).unwrap();

    assert_eq!(result, String::from("save"));
}
