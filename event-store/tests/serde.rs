use event_store::{CreateEvents, Event};

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Debug)]
struct ThingCreated {
    foo: String,
}

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Debug)]
struct ThingUpdated {
    name: String,
}

#[derive(CreateEvents, Debug)]
#[event_store(event_namespace = "some_ns", entity_type = "user")]
enum ExampleEvents {
    ThingCreated(Event<ThingCreated>),
    ThingUpdated(Event<ThingUpdated>),
}

#[test]
fn deserialize() {
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

    assert!(match out {
        ExampleEvents::ThingCreated(Event {
            data: Some(ThingCreated { foo }),
            ..
        }) => foo == String::from("Amazing"),
        _ => false,
    });
}

#[test]
fn serialize() {
    let out = serde_json::to_string(&serde_json::json!({
        "event_namespace": "some_ns",
        "event_type": "ThingUpdated",
        "entity_type": "user",
        "id": "89eba10f-7f15-48d2-b2a3-e4acb61d2f26",
        "sequence_number": 0,
        "entity_id": "89eba10f-7f15-48d2-b2a3-e4acb61d2f26",
        "created_at": "1985-04-12T23:20:50.52Z",
        "data": {
            "name": "Burning"
        },
        "context": {
            "subject_id": "89eba10f-7f15-48d2-b2a3-e4acb61d2f26",
            "hostname": "String",
            "username": "String"
        }
    }))
    .unwrap();

    assert_eq!(out, String::from(r#"{"context":{"hostname":"String","subject_id":"89eba10f-7f15-48d2-b2a3-e4acb61d2f26","username":"String"},"created_at":"1985-04-12T23:20:50.52Z","data":{"name":"Burning"},"entity_id":"89eba10f-7f15-48d2-b2a3-e4acb61d2f26","entity_type":"user","event_namespace":"some_ns","event_type":"ThingUpdated","id":"89eba10f-7f15-48d2-b2a3-e4acb61d2f26","sequence_number":0}"#));
}