extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate event_store_derive;
extern crate event_store;

use event_store::prelude::*;
use serde_json::{from_value, to_value};

#[test]
fn it_gets_event_name_and_namespace() {
    #[derive(EventData, Debug)]
    #[event_store(namespace = "test_thing")]
    struct TestEvent {
        thing: u32,
    }

    assert_eq!(
        TestEvent::event_namespace_and_type(),
        "test_thing.TestEvent"
    );

    assert_eq!(TestEvent::event_namespace(), "test_thing");

    assert_eq!(TestEvent::event_type(), "TestEvent");
}

#[test]
fn it_gets_renamed_event_name() {
    #[derive(EventData, Debug)]
    #[event_store(namespace = "test_thing")]
    #[event_store(rename = "OtherEvent")]
    struct TestEvent {
        thing: u32,
    }

    assert_eq!(
        TestEvent::event_namespace_and_type(),
        "test_thing.OtherEvent"
    );

    assert_eq!(TestEvent::event_type(), "OtherEvent");
}

#[test]
fn it_deserializes_combined_type_fields() {
    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    assert_eq!(
        from_value::<TestStruct>(json!({
            "type": "some_namespace.TestStruct",
            "thing": 100,
        }))
        .unwrap(),
        TestStruct { thing: 100 }
    );
}

#[test]
fn serialize_outputs_type_fields() {
    #[derive(EventData, Debug)]
    #[event_store(namespace = "test_thing")]
    struct TestEvent {
        thing: u32,
    }

    assert_eq!(
        TestEvent::event_namespace_and_type(),
        "test_thing.TestEvent"
    );
    assert_eq!(TestEvent::event_namespace(), "test_thing");
    assert_eq!(TestEvent::event_type(), "TestEvent");
}

#[test]
fn serialize_renamed_structs() {
    #[derive(EventData, Debug)]
    #[event_store(namespace = "test_thing")]
    #[event_store(rename = "A")]
    struct TestEvent {
        thing: u32,
    }

    assert_eq!(TestEvent::event_namespace_and_type(), "test_thing.A");
    assert_eq!(TestEvent::event_namespace(), "test_thing");
    assert_eq!(TestEvent::event_type(), "A");
}

#[test]
fn deserialize_renamed_structs() {
    #[derive(EventData, Debug, PartialEq)]
    #[event_store(namespace = "test_thing")]
    #[event_store(rename = "A")]
    struct TestEvent {
        thing: u32,
    }

    assert_eq!(
        from_value::<TestEvent>(json!({
            "type": "test_thing.A",
            "thing": 100,
        }))
        .unwrap(),
        TestEvent { thing: 100 }
    );
}

#[test]
fn roundtrip_renamed_struct() {
    #[derive(EventData, Debug, PartialEq)]
    #[event_store(namespace = "test_thing")]
    #[event_store(rename = "A")]
    struct TestEvent {
        thing: u32,
    }

    let input = TestEvent { thing: 99 };

    let encoded = to_value(&input).expect("Could not encode");

    let output = from_value::<TestEvent>(encoded.clone()).expect("Could not decode");

    assert_eq!(
        encoded,
        json!({
            "type": "test_thing.A",
            "event_type": "A",
            "event_namespace": "test_thing",
            "thing": 99,
        }),
    );

    assert_eq!(output, TestEvent { thing: 99 });
}
