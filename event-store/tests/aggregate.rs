extern crate event_store;
extern crate r2d2;
extern crate r2d2_postgres;

use event_store::prelude::*;
use event_store::testhelpers::{
    TestCounterEntity, TestDecrementEvent, TestEvents, TestIncrementEvent,
};
use event_store::Event;

#[test]
fn it_aggregates_events() {
    let events = vec![
        Event::from_data(TestEvents::Inc(TestIncrementEvent {
            by: 1,
            ident: "it_aggregates_events".into(),
        })),
        Event::from_data(TestEvents::Inc(TestIncrementEvent {
            by: 1,
            ident: "it_aggregates_events".into(),
        })),
        Event::from_data(TestEvents::Dec(TestDecrementEvent {
            by: 2,
            ident: "it_aggregates_events".into(),
        })),
        Event::from_data(TestEvents::Inc(TestIncrementEvent {
            by: 2,
            ident: "it_aggregates_events".into(),
        })),
        Event::from_data(TestEvents::Dec(TestDecrementEvent {
            by: 3,
            ident: "it_aggregates_events".into(),
        })),
        Event::from_data(TestEvents::Dec(TestDecrementEvent {
            by: 3,
            ident: "it_aggregates_events".into(),
        })),
    ];

    let result: TestCounterEntity = events
        .iter()
        .fold(TestCounterEntity::default(), TestCounterEntity::apply_event);

    assert_eq!(result, TestCounterEntity { counter: -4 });
}
