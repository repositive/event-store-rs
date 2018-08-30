extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate event_store_derive;
extern crate event_store;

use event_store::EventData;
use serde_json::from_value;
use serde_json::to_value;

#[test]
fn it_deserializes_events_correctly() {
    #[derive(EventData, Debug)]
    #[event_store(namespace = "remote_ns_newer")]
    struct TestEvent {
        thing: u32,
    }

    let json = json!({
            "type": "remote_ns.TestEventOld",
            "event_namespace": "remote_ns_newer",
            "event_type": "TestEvent",
            "thing": 100,
        });

    let event = from_value::<TestEvent>(json);

    assert!(event.is_ok());
    assert_eq!(event.unwrap().thing, 100);
}

#[test]
fn it_deserializes_events_with_old_def() {
    #[derive(EventData, Debug)]
    #[event_store(namespace = "remote_ns")]
    struct TestEvent {
        thing: u32,
    }

    let json = json!({
            "type": "remote_ns.TestEvent",
            "thing": 100,
        });

    let event = from_value::<TestEvent>(json);

    assert!(event.is_ok());
    assert_eq!(event.unwrap().thing, 100);
}

#[test]
fn it_deserializes_events_with_new_def() {
    #[derive(EventData, Debug)]
    #[event_store(namespace = "remote_ns_newer")]
    struct TestEvent {
        thing: u32,
    }

    let json = json!({
            "event_namespace": "remote_ns_newer",
            "event_type": "TestEvent",
            "thing": 100,
        });

    let event = from_value::<TestEvent>(json);

    assert!(event.is_ok());
    assert_eq!(event.unwrap().thing, 100);
}

#[test]
fn it_gets_a_namespaced_struct_type() {
    #[derive(EventData)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    let thing = TestStruct { thing: 100 };

    assert_eq!(
        thing.event_namespace_and_type(),
        "some_namespace.TestStruct"
    );
}

#[test]
fn it_serializes_structs() {
    #[derive(EventData)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    let thing = TestStruct { thing: 100 };

    assert_eq!(
        to_value(&thing).unwrap(),
        json!({
            "type": "some_namespace.TestStruct",
            "event_namespace": "some_namespace",
            "event_type": "TestStruct",
            "thing": 100
        })
    );
}
