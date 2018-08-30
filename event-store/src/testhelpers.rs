//! Test helpers. Do not use in application code.

use super::{Aggregator, Event};
use adapters::PgQuery;
use postgres::types::ToSql;

#[derive(EventData, Debug)]
#[event_store(namespace = "some_namespace")]
/// Test event
pub struct TestIncrementEvent {
    /// Increment by this much
    pub by: i32,

    /// Test identifier
    pub ident: String,
}

#[derive(EventData, Debug)]
#[event_store(namespace = "some_namespace")]
/// Test event
pub struct TestDecrementEvent {
    /// Decrement by this much
    pub by: i32,

    /// Test identifier
    pub ident: String,
}

#[derive(EventData, Debug)]
#[event_store(namespace = "some_namespace")]
/// Set of all events in the domain
pub enum TestEvents {
    /// Increment
    Inc(TestIncrementEvent),
    /// Decrement
    Dec(TestDecrementEvent),
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
    fn apply_event(acc: Self, event: &Event<TestEvents>) -> Self {
        let counter = match event.data {
            TestEvents::Inc(ref inc) => acc.counter + inc.by,
            TestEvents::Dec(ref dec) => acc.counter - dec.by,
        };

        Self { counter, ..acc }
    }

    fn query(field: String) -> PgQuery<'a> {
        let mut params: Vec<Box<ToSql>> = Vec::new();

        params.push(Box::new(field));

        PgQuery::new("select * from events where data->>'ident' = $1", params)
    }
}
