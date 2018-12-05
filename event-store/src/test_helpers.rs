use crate::aggregator::Aggregator;
use crate::event::Event;
use crate::pg::PgQuery;
use postgres::types::ToSql;

/// Set of all events in the domain
#[derive(Events, Debug)]
pub enum TestEvents {
    Inc(Event<TestEvent>),
}

#[derive(EventData, Debug)]
#[event_store(namespace = "some_namespace")]
pub struct TestEvent {
    pub num: i32,
}

/// Testing entity for a pretend domain
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct TestCounterEntity {
    /// Current counter value
    pub counter: i32,
}

impl Default for TestCounterEntity {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

impl Aggregator<TestEvents, String, PgQuery> for TestCounterEntity {
    fn apply_event(acc: Self, event: &TestEvents) -> Self {
        let counter = match event {
            TestEvents::Inc(ref inc) => acc.counter + inc.data.num,
        };

        Self { counter, ..acc }
    }

    fn query(_query_args: String) -> PgQuery {
        let params: Vec<Box<ToSql>> = Vec::new();

        PgQuery::new("select * from events", params)
    }
}
