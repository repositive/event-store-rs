#[macro_use]
extern crate criterion;
extern crate event_store;

use criterion::Criterion;
use event_store::testhelpers::{
    TestCounterEntity, TestDecrementEvent, TestEvents, TestIncrementEvent,
};
use event_store::{Aggregator, Event, EventData};

fn aggregate_from_default(c: &mut Criterion) {
    c.bench_function("aggregate from default", move |b| {
        b.iter(|| {
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

            let _result: TestCounterEntity = events
                .iter()
                .fold(TestCounterEntity::default(), TestCounterEntity::apply_event);
        })
    });
}

fn event_name_from_object(c: &mut Criterion) {
    c.bench_function("get event name from event object", move |b| {
        b.iter(|| {
            let event = Event::from_data(TestEvents::Inc(TestIncrementEvent {
                by: 1,
                ident: "it_aggregates_events".into(),
            }));

            event.data().event_type();
        })
    });
}

criterion_group!(testhelpers, aggregate_from_default, event_name_from_object);
criterion_main!(testhelpers);
