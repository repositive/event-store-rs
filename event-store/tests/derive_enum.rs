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
fn it_deserializes_combined_type_fields() {
    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    #[derive(Events, PartialEq, Debug)]
    enum TestEnum {
        TestStruct(TestStruct),
    }

    assert_eq!(
        from_value::<TestEnum>(json!({
            "type": "some_namespace.TestStruct",
            "thing": 100,
        })).unwrap(),
        TestEnum::TestStruct(TestStruct { thing: 100 })
    );
}

#[test]
fn it_overrides_combined_fields() {
    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    struct OtherStruct {
        thing: u32,
    }

    #[derive(Events, PartialEq, Debug)]
    enum TestEnum {
        TestStruct(TestStruct),
        OtherStruct(OtherStruct),
    }

    assert_eq!(
        from_value::<TestEnum>(json!({
            "type": "some_namespace.TestStruct",
            "event_type": "OtherStruct",
            "event_namespace": "some_namespace",
            "thing": 100,
        })).unwrap(),
        TestEnum::OtherStruct(OtherStruct { thing: 100 })
    );
}

#[test]
fn it_deserializes_enums() {
    #[derive(EventData, PartialEq, Debug)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    #[derive(Events, PartialEq, Debug)]
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

    #[derive(Events, PartialEq, Debug)]
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
    #[derive(EventData, PartialEq, Debug, Clone)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    #[derive(Events, PartialEq, Debug, Clone)]
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
            "type": "some_namespace.TestStruct",
            "event_namespace": "some_namespace",
            "event_type": "TestStruct",
            "thing": 100,
        })
    );
}

#[test]
fn it_roundtrips_combined_type_field() {
    #[derive(EventData, PartialEq, Debug, Clone)]
    #[event_store(namespace = "some_namespace")]
    struct TestStruct {
        thing: u32,
    }

    #[derive(Events, PartialEq, Debug, Clone)]
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
            "type": "some_namespace.TestStruct",
            "event_namespace": "some_namespace",
            "event_type": "TestStruct",
            "thing": 100,
        })
    );
}

#[test]
fn it_differentiates_structs_with_same_shape() {
    #[derive(EventData, PartialEq, Debug, Clone)]
    #[event_store(namespace = "some_namespace")]
    struct ThingA {
        foo: u32,
    };
    #[derive(EventData, PartialEq, Debug, Clone)]
    #[event_store(namespace = "some_namespace")]
    struct ThingB {
        foo: u32,
    };
    #[derive(EventData, PartialEq, Debug, Clone)]
    #[event_store(namespace = "some_namespace")]
    struct ThingC {
        bar: u8,
    };

    #[derive(Events, PartialEq, Debug, Clone)]
    enum TestEnum {
        A(ThingA),
        B(ThingB),
        C(ThingC),
    }

    let v: TestEnum = from_value(json!({
        "event_type": "ThingB",
        "event_namespace": "some_namespace",
        "foo": 100
    })).unwrap();

    assert_eq!(v, TestEnum::B(ThingB { foo: 100 }));
}
