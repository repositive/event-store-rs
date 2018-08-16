use super::{Aggregator, Event, Events, PgQuery};
use postgres::types::ToSql;

#[derive(Deserialize)]
pub struct TestIncrementEvent {
    pub by: i32,
}
#[derive(Deserialize)]
pub struct TestDecrementEvent {
    pub by: i32,
}

impl Event for TestIncrementEvent {}
impl Event for TestDecrementEvent {}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum TestEvents {
    #[serde(rename = "some_namespace.Inc")]
    Inc(TestIncrementEvent),
    #[serde(rename = "some_namespace.Dec")]
    Dec(TestDecrementEvent),
    #[serde(rename = "some_namespace.Other")]
    Other,
}

impl Events for TestEvents {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TestCounterEntity {
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
            TestEvents::Inc(inc) => acc.counter + inc.by,
            TestEvents::Dec(dec) => acc.counter - dec.by,
            _ => acc.counter,
        };

        Self { counter, ..acc }
    }

    fn query(field: String) -> PgQuery<'a> {
        let mut params: Vec<Box<ToSql>> = Vec::new();

        params.push(Box::new(field));

        PgQuery::new(
            "select * from events where data->>'test_field' = $1",
            params,
        )
    }
}
