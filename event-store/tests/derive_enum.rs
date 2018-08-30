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

// #[test]
// fn it_gets_namespaced_event_names() {
//     let event_a = Events::EnumEventA(EventA { thing: 100 });
//     let event_b = Events::EnumEventB(EventB { thing: 100 });
//     let event_c = Events::EnumNsEventC(NsEventC { thing: 100 });

//     assert_eq!(event_a.event_namespace_and_type(), "test_ns.EnumEventA");
//     assert_eq!(event_b.event_namespace_and_type(), "test_ns.EnumEventB");
//     assert_eq!(event_c.event_namespace_and_type(), "remote_ns.EnumNsEventC");
// }

// #[test]
// fn it_serializes_events_with_extra_fields() {
//     let event = Events::EnumEventA(EventA { thing: 100 });

//     let json = to_value(&event).unwrap();

//     assert_eq!(
//         json,
//         json!({
//             "type": "test_ns.EventA",
//             "event_namespace": "test_ns",
//             "event_type": "EventA",
//             "thing": 100,
//         })
//     );
// }

// #[test]
// fn it_serializes_events_with_overridden_namespace() {
//     let event = Events::EnumNsEventC(NsEventC { thing: 100 });

//     let json = to_value(&event).unwrap();

//     assert_eq!(
//         json,
//         json!({
//             "type": "remote_ns.RenamedRemoteEvent",
//             "event_namespace": "remote_ns",
//             "event_type": "RenamedRemoteEvent",
//             "thing": 100,
//         })
//     );
// }
