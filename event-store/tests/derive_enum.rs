extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate event_store_derive;
extern crate event_store;

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
        Variant(TestStruct),
    }

    let event = TestEnum::Variant(TestStruct { thing: 100 });
    let encoded = to_value(event.clone()).expect("Failed to encode");
    let decoded: TestEnum = from_value(encoded.clone()).expect("Failed to decode");

    assert_eq!(event, decoded.clone());
    assert_eq!(
        encoded,
        json!({
            "type": "some_namespace.Variant",
            "event_namespace": "some_namespace",
            "event_type": "Variant",
            "thing": 100,
        })
    );
}

#[test]
fn it_roundtrips_overridden_namespaces() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct TestStruct {
        thing: u32,
    }

    #[derive(Events, PartialEq, Debug, Clone)]
    #[event_store(namespace = "some_namespace")]
    enum TestEnum {
        #[event_store(namespace = "other_ns")]
        Variant(TestStruct),
    }

    let event = TestEnum::Variant(TestStruct { thing: 100 });
    let encoded = to_value(event.clone()).expect("Failed to encode");
    let decoded: TestEnum = from_value(encoded.clone()).expect("Failed to decode");

    assert_eq!(event, decoded.clone());
    assert_eq!(
        encoded,
        json!({
            "type": "other_ns.Variant",
            "event_namespace": "other_ns",
            "event_type": "Variant",
            "thing": 100,
        })
    );
}

#[test]
fn it_roundtrips_renamed_variants() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct TestStruct {
        thing: u32,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct OtherStruct {
        foo: u32,
    }

    #[derive(Events, PartialEq, Debug, Clone)]
    #[event_store(namespace = "some_namespace")]
    enum TestEnum {
        #[event_store(rename = "RenamedTestStruct")]
        Variant(TestStruct),
        A(OtherStruct),
    }

    let event = TestEnum::Variant(TestStruct { thing: 100 });
    let encoded = to_value(event.clone()).expect("Failed to encode");
    let decoded: TestEnum = from_value(encoded.clone()).expect("Failed to decode");

    assert_eq!(event, decoded.clone());
    assert_eq!(
        encoded,
        json!({
            "type": "some_namespace.RenamedTestStruct",
            "event_namespace": "some_namespace",
            "event_type": "RenamedTestStruct",
            "thing": 100,
        })
    );
}
