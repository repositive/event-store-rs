//! Test helpers. Do not use in application code.

use super::{Aggregator, Event, EventData};
use adapters::PgQuery;
use postgres::types::ToSql;

#[derive(Serialize, Deserialize)]
/// Test event
pub struct TestIncrementEvent {
    /// Increment by this much
    pub by: i32,

    /// Test identifier
    pub ident: String,
}

#[derive(Serialize, Deserialize)]
/// Test event
pub struct TestDecrementEvent {
    /// Decrement by this much
    pub by: i32,

    /// Test identifier
    pub ident: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
/// Set of all events in the domain
pub enum TestEvents {
    /// Increment
    #[serde(rename = "some_namespace.Inc")]
    Inc(TestIncrementEvent),
    /// Decrement
    #[serde(rename = "some_namespace.Dec")]
    Dec(TestDecrementEvent),
    /// Some other event
    #[serde(rename = "some_namespace.Other")]
    Other,
}

impl EventData for TestEvents {
    fn event_type(&self) -> String {
        match &self {
            TestEvents::Inc(_) => "some_namespace.Inc",
            TestEvents::Dec(_) => "some_namespace.Dec",
        }.into()
    }
}

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
            _ => acc.counter,
        };

        Self { counter, ..acc }
    }

    fn query(field: String) -> PgQuery<'a> {
        let mut params: Vec<Box<ToSql>> = Vec::new();

        params.push(Box::new(field));

        PgQuery::new("select * from events where data->>'ident' = $1", params)
    }
}
