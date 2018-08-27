#[macro_use]
extern crate criterion;
extern crate event_store_rs;

use criterion::Criterion;
use event_store_rs::testhelpers::{
    TestCounterEntity, TestDecrementEvent, TestEvents, TestIncrementEvent,
};
use event_store_rs::Aggregator;

fn aggregate_from_default(c: &mut Criterion) {
    c.bench_function("aggregate from default", move |b| {
        b.iter(|| {
            let events = vec![
                TestEvents::Inc(TestIncrementEvent {
                    by: 1,
                    ident: "it_aggregates_events".into(),
                }),
                TestEvents::Inc(TestIncrementEvent {
                    by: 1,
                    ident: "it_aggregates_events".into(),
                }),
                TestEvents::Dec(TestDecrementEvent {
                    by: 2,
                    ident: "it_aggregates_events".into(),
                }),
                TestEvents::Inc(TestIncrementEvent {
                    by: 2,
                    ident: "it_aggregates_events".into(),
                }),
                TestEvents::Dec(TestDecrementEvent {
                    by: 3,
                    ident: "it_aggregates_events".into(),
                }),
                TestEvents::Dec(TestDecrementEvent {
                    by: 3,
                    ident: "it_aggregates_events".into(),
                }),
            ];

            let _result: TestCounterEntity = events
                .iter()
                .fold(TestCounterEntity::default(), TestCounterEntity::apply_event);
        })
    });
}

criterion_group!(testhelpers, aggregate_from_default);
criterion_main!(testhelpers);
