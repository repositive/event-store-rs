extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate event_store_derive;
extern crate event_store;
// TODO: Figure out how to remove this
extern crate event_store_derive_internals;

use event_store::EventData;
use serde_json::from_value;
use serde_json::to_value;

#[test]
fn it_deserializes_enums() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStruct {
        thing: u32,
    }

    #[derive(Events, PartialEq, Debug)]
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
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStruct {
        thing: u32,
    }

    #[derive(Events, PartialEq, Debug)]
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
fn it_roundtrips() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct TestStruct {
        thing: u32,
    }

    #[derive(Events, PartialEq, Debug, Clone)]
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