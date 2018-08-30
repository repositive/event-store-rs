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
fn it_deserializes_enums() {
    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    enum TestEnum {
        TestStruct(TestStruct),
    }

    assert_eq!(
        from_value::<TestEnum>(json!({
            "event_namespace": "some_namespace",
            "event_type": "TestStruct",
            "thing": 100,
        })).unwrap(),
        TestEnum::TestStruct(TestStruct { thing: 100 })
    );
}

#[test]
fn it_serializes_enums() {
    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    enum TestEnum {
        TestStruct(TestStruct),
    }

    assert_eq!(
        to_value(TestEnum::TestStruct(TestStruct { thing: 100 })).unwrap(),
        json!({
            "type": "some_namespace.TestStruct",
            "event_namespace": "some_namespace",
            "event_type": "TestStruct",
            "thing": 100,
        }),
    );
}

#[test]
fn it_gets_the_namespace_and_type() {
    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    enum TestEnum {
        TestStruct(TestStruct),
    }

    let thing = TestEnum::TestStruct(TestStruct { thing: 100 });

    assert_eq!(
        thing.event_namespace_and_type(),
        "some_namespace.TestStruct"
    );

    assert_eq!(thing.event_namespace(), "some_namespace");

    assert_eq!(thing.event_type(), "TestStruct");
}

#[test]
#[ignore]
fn it_gets_the_overridden_namespace() {
    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    enum TestEnum {
        // TODO: Make this work
        // #[event_store(namespace = "overridden")]
        TestStruct(TestStruct),
    }

    let thing = TestEnum::TestStruct(TestStruct { thing: 100 });

    assert_eq!(thing.event_namespace_and_type(), "overridden.TestStruct");

    assert_eq!(thing.event_namespace(), "overridden");

    assert_eq!(thing.event_type(), "TestStruct");
}

#[test]
fn it_roundtrips() {
    #[derive(EventData, PartialEq, Debug, Clone)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    #[derive(EventData, PartialEq, Debug, Clone)]
    #[event_store(namespace = "some_namespace")]
    enum TestEnum {
        TestStruct(TestStruct),
    }

    let event = TestEnum::TestStruct(TestStruct { thing: 100 });

    let encoded = to_value(event.clone()).expect("Failed to encode");

    let decoded: TestEnum = from_value(encoded.clone()).expect("Failed to decode");

    assert_eq!(event, decoded.clone());
    assert_eq!(
        encoded,
        json!({
            "type": "some_namespace.TestStruct",
            "event_namespace": "some_namespace",
            "event_type": "TestStruct",
            "thing": 100,
        })
    );
}
