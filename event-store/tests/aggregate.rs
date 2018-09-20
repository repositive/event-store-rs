extern crate event_store;
extern crate r2d2;
extern crate r2d2_postgres;

use event_store::adapters::StubQuery;
use event_store::prelude::*;
use event_store::testhelpers::{
    TestCounterEntity, TestDecrementEvent, TestEvents, TestIncrementEvent,
};
use event_store::Event;

#[test]
fn it_aggregates_events() {
    let events = vec![
        TestEvents::Inc(Event::from_data(TestIncrementEvent {
            by: 1,
            ident: "it_aggregates_events".into(),
        })),
        TestEvents::Inc(Event::from_data(TestIncrementEvent {
            by: 1,
            ident: "it_aggregates_events".into(),
        })),
        TestEvents::Dec(Event::from_data(TestDecrementEvent {
            by: 2,
            ident: "it_aggregates_events".into(),
        })),
        TestEvents::Inc(Event::from_data(TestIncrementEvent {
            by: 2,
            ident: "it_aggregates_events".into(),
        })),
        TestEvents::Dec(Event::from_data(TestDecrementEvent {
            by: 3,
            ident: "it_aggregates_events".into(),
        })),
        TestEvents::Dec(Event::from_data(TestDecrementEvent {
            by: 3,
            ident: "it_aggregates_events".into(),
        })),
    ];

    fn agg<T>(events: Vec<TestEvents>) -> T
    where
        T: Default + Aggregator<TestEvents, String, StubQuery>,
    {
        events.iter().fold(T::default(), T::apply_event)
    }

    let result = agg::<TestCounterEntity>(events);

    assert_eq!(result, TestCounterEntity { counter: -4 });
}
