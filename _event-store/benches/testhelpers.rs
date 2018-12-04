#[macro_use]
extern crate criterion;
extern crate event_store;
extern crate serde_json;

use criterion::Criterion;
use event_store::prelude::*;
use event_store::testhelpers::{
    TestCounterEntity, TestDecrementEvent, TestEvents, TestIncrementEvent,
};
use event_store::Event;
use serde_json::{from_str, to_string, to_value, Value};
use std::time::Duration;

fn aggregate_from_default(c: &mut Criterion) {
    c.bench_function("aggregate from default", move |b| {
        b.iter(|| {
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

            let _result: TestCounterEntity = events
                .iter()
                .fold(TestCounterEntity::default(), TestCounterEntity::apply_event);
        })
    });
}

fn serialize_event(c: &mut Criterion) {
    c.bench_function("serialize event", move |b| {
        b.iter(|| {
            let event = TestEvents::Inc(Event::from_data(TestIncrementEvent {
                by: 1,
                ident: "serialize_event".into(),
            }));

            let _json: Value = to_value(event).unwrap();
        })
    });
}

fn deserialize_event(c: &mut Criterion) {
    let incoming_str = to_string(&TestEvents::Inc(Event::from_data(TestIncrementEvent {
        by: 1,
        ident: "serialize_event".into(),
    })))
    .expect("Could not create test event JSON");

    c.bench_function("deserialize event", move |b| {
        b.iter(|| {
            let _event: TestEvents = from_str(&incoming_str).unwrap();
        })
    });
}

fn roundtrip(c: &mut Criterion) {
    c.bench_function("serialize deserialize roundtrip", move |b| {
        b.iter(|| {
            let event = TestEvents::Inc(Event::from_data(TestIncrementEvent {
                by: 1,
                ident: "serialize_event".into(),
            }));

            let json = to_string(&event).unwrap();

            let _decoded: TestEvents = from_str(&json).unwrap();
        })
    });
}

criterion_group!(
    name = testhelpers;

    config = Criterion::default()
        .warm_up_time(Duration::from_millis(1000))
        .sample_size(100)
        .measurement_time(Duration::from_millis(4000));

    targets =
        aggregate_from_default,
        serialize_event,
        deserialize_event,
        roundtrip
);
criterion_main!(testhelpers);
