#[macro_use]
extern crate criterion;

use chrono::{DateTime, Utc};
use criterion::{Benchmark, Criterion, Throughput};
use event_store::{Context, CreateEvents, Event};
use std::net::{IpAddr, Ipv4Addr};
use uuid::Uuid;

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Debug)]
struct ThingCreated {
    foo: String,
}

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Debug)]
struct ThingUpdated {
    bar: u32,
    baz: u64,
    quux: Uuid,
    some_time: DateTime<Utc>,
}

// TODO: Support default namespace on enum
// TODO: Support default entity type on enum
// TODO: Support leaving out `event_type` attrib and gleaning it from `Event<D>`
#[derive(CreateEvents, Debug)]
#[event_store(event_namespace = "some_ns", entity_type = "user")]
enum ExampleEvents {
    // TODO: Validate casing of attributes
    // TODO: Determine event type from enum variant by default
    #[event_store(event_type = "ThingCreated")]
    ThingCreated(Event<ThingCreated>),

    #[event_store(event_type = "ThingUpdated")]
    ThingUpdated(Event<ThingUpdated>),
}

fn serialize_string_data(c: &mut Criterion) {
    let event = ExampleEvents::ThingCreated(Event {
        id: Uuid::new_v4(),
        sequence_number: None,
        created_at: Utc::now(),
        entity_id: Uuid::new_v4(),
        data: Some(ThingCreated {
            foo: String::from("Amazing"),
        }),
        context: Context {
            purge: None,
            hostname: String::new(),
            username: String::new(),
            ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            subject_id: Uuid::new_v4(),
        },
    });

    c.bench(
        "serialize",
        Benchmark::new("string data", move |b| {
            b.iter(|| serde_json::to_string(&event))
        })
        .throughput(Throughput::Elements(1u32)),
    );
}

fn deserialize_string_data(c: &mut Criterion) {
    let input = r#"{"event_namespace":"some_ns","entity_type":"user","event_type":"ThingCreated","id":"30506604-fc47-4d7e-9388-fd34c67d24eb","sequence_number":null,"entity_id":"122c937f-3979-487f-95eb-272160400740","created_at":"2019-05-26T11:54:39.822350426Z","data":{"foo":"Amazing"},"context":{"subject_id":"6c2dc5e5-7a69-4363-a830-03406e29d439","hostname":"","username":"","ip":"127.0.0.1","purge":null}}"#;

    c.bench(
        "deserialize",
        Benchmark::new("string data", move |b| {
            b.iter(|| serde_json::from_str::<ExampleEvents>(input))
        })
        .throughput(Throughput::Elements(1u32)),
    );
}

criterion_group!(benches, serialize_string_data, deserialize_string_data);
criterion_main!(benches);
