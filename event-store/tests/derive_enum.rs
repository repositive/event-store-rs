extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate event_store_derive;
extern crate event_store;

use event_store::Events;
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
            "event_namespace": "some_namespace",
            "event_type": "RenamedTestStruct",
            "thing": 100,
        })
    );
}

#[test]
fn it_gets_variant_strings() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct ThingA;
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct ThingB;
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct ThingC;
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct ThingD;

    #[derive(Events, PartialEq, Debug, Clone)]
    #[event_store(namespace = "some_namespace")]
    enum TestEnum {
        A(ThingA),
        #[event_store(rename = "RenamedB")]
        B(ThingB),
        #[event_store(namespace = "other_ns")]
        #[event_store(rename = "RenamedC")]
        C(ThingC),
        #[event_store(namespace = "other_ns")]
        D(ThingD),
    }

    let cases = vec![
        (
            TestEnum::A(ThingA),
            "some_namespace.A",
            "some_namespace",
            "A",
        ),
        (
            TestEnum::B(ThingB),
            "some_namespace.RenamedB",
            "some_namespace",
            "RenamedB",
        ),
        (
            TestEnum::C(ThingC),
            "other_ns.RenamedC",
            "other_ns",
            "RenamedC",
        ),
        (TestEnum::D(ThingD), "other_ns.D", "other_ns", "D"),
    ];

    for (variant, expected_ns_and_ty, expected_ns, expected_ty) in cases {
        assert_eq!(variant.event_namespace_and_type(), expected_ns_and_ty);
        assert_eq!(variant.event_namespace(), expected_ns);
        assert_eq!(variant.event_type(), expected_ty);
    }
}

#[test]
fn it_differentiates_structs_with_same_shape() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct ThingA {
        foo: u32,
    };
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct ThingB {
        foo: u32,
    };
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct ThingC {
        bar: u8,
    };

    #[derive(Events, PartialEq, Debug, Clone)]
    #[event_store(namespace = "some_namespace")]
    enum TestEnum {
        A(ThingA),
        B(ThingB),
        C(ThingC),
    }

    let v: TestEnum = from_value(json!({
        "event_type": "B",
        "event_namespace": "some_namespace",
        "foo": 100
    })).unwrap();

    assert_eq!(v, TestEnum::B(ThingB { foo: 100 }));
}
