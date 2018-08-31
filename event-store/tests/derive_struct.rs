extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate event_store_derive;
extern crate event_store;

use event_store::EventData;

#[test]
fn it_gets_event_name_and_namespace() {
    #[derive(Serialize, Deserialize, Debug)]
    struct TestEvent {
        thing: u32,
    }

    #[derive(Events)]
    #[event_store(namespace = "test_thing")]
    enum TestEvents {
        A(TestEvent),
    }

    assert_eq!(TestEvent::event_namespace_and_type(), "test_thing.A");

    assert_eq!(TestEvent::event_namespace(), "test_thing");

    assert_eq!(TestEvent::event_type(), "A");
}
