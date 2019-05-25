use event_store::{CreateEvents, Event, EventStore};
use postgres::{Client, NoTls};
use r2d2_postgres::PostgresConnectionManager;

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Debug)]
struct ThingCreated {
    foo: String,
}

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Debug)]
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

// #[derive(CreateEvents, Debug)]
// #[event_store(event_namespace = "another_ns", entity_type = "dumpster")]
// struct FloobCreated {
//     name: String,
// }

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

    let ser = serde_json::to_string(&out).unwrap();

    println!("Ser {}", ser);

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

    // let out: Event<FloobCreated> = serde_json::from_value(serde_json::json!({
    //     "event_namespace": "another_ns",
    //     "event_type": "FloobCreated",
    //     "entity_type": "dumpster",
    //     "id": "89eba10f-7f15-48d2-b2a3-e4acb61d2f26",
    //     "sequence_number": 0,
    //     "entity_id": "89eba10f-7f15-48d2-b2a3-e4acb61d2f26",
    //     "created_at": "1985-04-12T23:20:50.52Z",
    //     "data": {
    //         "name": "Burning"
    //     },
    //     "context": {
    //         "subject_id": "89eba10f-7f15-48d2-b2a3-e4acb61d2f26",
    //         "hostname": "String",
    //         "username": "String"
    //     }
    // }))
    // .unwrap();

    // println!("Floob: {:?}", out);
}
