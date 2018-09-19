//! Test helpers. Do not use in application code.

use adapters::PgQuery;
use adapters::{PgCacheAdapter, PgStoreAdapter, StubEmitterAdapter};
use prelude::*;
use r2d2;
use r2d2::Pool;
use r2d2_postgres::postgres::types::ToSql;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use Event;
use EventStore;

/// Test event
#[derive(EventData, Debug)]
#[event_store(namespace = "some_namespace")]
pub struct TestIncrementEvent {
    /// Increment by this much
    pub by: i32,

    /// Test identifier
    pub ident: String,
}

/// Test event
#[derive(EventData, Debug)]
#[event_store(namespace = "some_namespace")]
pub struct TestDecrementEvent {
    /// Decrement by this much
    pub by: i32,

    /// Test identifier
    pub ident: String,
}

#[derive(Events, Debug)]
/// Set of all events in the domain
pub enum TestEvents {
    /// Increment
    Inc(Event<TestIncrementEvent>),
    /// Decrement
    Dec(Event<TestDecrementEvent>),
}

// impl EventData for TestEvents {}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
/// Testing entity for a pretend domain
pub struct TestCounterEntity {
    /// Current counter value
    pub counter: i32,
}

impl Default for TestCounterEntity {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

impl<'a> Aggregator<TestEvents, String, PgQuery<'a>> for TestCounterEntity {
    fn apply_event(acc: Self, event: &TestEvents) -> Self {
        let counter = match event {
            TestEvents::Inc(ref inc) => acc.counter + inc.data.by,
            TestEvents::Dec(ref dec) => acc.counter - dec.data.by,
        };

        Self { counter, ..acc }
    }

    fn query(field: String) -> PgQuery<'a> {
        let mut params: Vec<Box<ToSql + Send + Sync>> = Vec::new();

        params.push(Box::new(field));

        PgQuery::new("select * from events where data->>'ident' = $1", params)
    }
}

/// Create an event store from a Postgres connection pool
#[macro_export]
macro_rules! pg_store {
    ($conn:ident) => {{
        let store_adapter = PgStoreAdapter::new($conn.clone());
        let cache_adapter = PgCacheAdapter::new($conn.clone());
        let emitter_adapter = StubEmitterAdapter::new();

        EventStore::new(store_adapter, cache_adapter, emitter_adapter)
    }};
}

/// Delete all `TestEvent` events matching an identifier from the `events` table
#[macro_export]
macro_rules! pg_delete_events {
    ($conn:ident, $ident:expr) => {{
        $conn
            .get()
            .unwrap()
            .execute("DELETE FROM events WHERE data->>'ident' = $1", &[&$ident])
            .expect("Failed to delete events");
    }};
}

/// Remove EVERY entry from the `aggregate_cache` table
#[macro_export]
macro_rules! pg_empty_cache {
    ($conn:ident) => {{
        $conn
            .get()
            .unwrap()
            .execute("TRUNCATE aggregate_cache", &[])
            .expect("Failed to trunacte cache table");
    }};
}
