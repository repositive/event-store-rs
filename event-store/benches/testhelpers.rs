#[macro_use]
extern crate criterion;
extern crate event_store;
#[macro_use]
extern crate serde_json;

use criterion::Criterion;
use event_store::testhelpers::{
    TestCounterEntity, TestDecrementEvent, TestEvents, TestIncrementEvent,
};
use event_store::{Aggregator, Event};
use serde_json::{from_str, to_string, to_value, Value};
use std::time::Duration;

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

fn serialize_event(c: &mut Criterion) {
    c.bench_function("serialize event", move |b| {
        b.iter(|| {
            let event = Event::from_data(TestEvents::Inc(TestIncrementEvent {
                by: 1,
                ident: "serialize_event".into(),
            }));

            let _json: Value = to_value(event).unwrap();
        })
    });
}

fn deserialize_event(c: &mut Criterion) {
    let incoming_str = to_string(&json!({
        "event_namespace": "some_namespace",
        "event_type": "Inc",
        "by": 1,
        "ident": "deserialize_event"
    })).expect("Could not create test event JSON");

    c.bench_function("deserialize event", move |b| {
        b.iter(|| {
            let _event: TestEvents = from_str(&incoming_str).unwrap();
        })
    });
}

fn roundtrip(c: &mut Criterion) {
    c.bench_function("serialize deserialize roundtrip", move |b| {
        b.iter(|| {
            let event = TestEvents::Inc(TestIncrementEvent {
                by: 1,
                ident: "serialize_event".into(),
            });

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
