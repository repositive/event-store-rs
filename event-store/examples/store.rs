use event_store::{CreateEvents, Event, EventStore};
use postgres::{Client, NoTls};
use r2d2_postgres::PostgresConnectionManager;

#[derive(serde_derive::Deserialize, Debug)]
struct ThingCreated {
    foo: String,
}

#[derive(serde_derive::Deserialize, Debug)]
struct ThingUpdated {
    bar: u32,
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

fn main() {
    println!("Example");

    let input = serde_json::json!({
        "event_namespace": "some_ns",
        "event_type": "ThingCreated",
        "entity_type": "user",
        "id": "89eba10f-7f15-48d2-b2a3-e4acb61d2f26",
        "sequence_number": 0,
        "entity_id": "89eba10f-7f15-48d2-b2a3-e4acb61d2f26",
        "created_at": "1985-04-12T23:20:50.52Z",
        "data": {
            "foo": "Amazing"
        },
        "context": {
            "subject_id": "89eba10f-7f15-48d2-b2a3-e4acb61d2f26",
            "hostname": "String",
            "username": "String"
        }
    });

    let out: ExampleEvents = serde_json::from_value(input).unwrap();

    println!(
        "Foo: {:?}",
        match out {
            ExampleEvents::ThingCreated(Event {
                data: Some(ThingCreated { foo }),
                ..
            }) => foo,
            _ => String::from("Unknown"),
        }
    );
}
