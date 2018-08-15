#[macro_use]
extern crate serde_derive;
extern crate event_store_rs;
extern crate serde;
extern crate serde_json;

use event_store_rs::{Aggregator, Event, Events, PgQuery, PgStore};

#[derive(Deserialize)]
struct Increment {
    pub by: i32,
}
#[derive(Deserialize)]
struct Decrement {
    pub by: i32,
}

impl Event for Increment {}
impl Event for Decrement {}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ToasterEvents {
    #[serde(rename = "some_namespace.Inc")]
    Inc(Increment),
    #[serde(rename = "some_namespace.Dec")]
    Dec(Decrement),
    #[serde(rename = "some_namespace.Other")]
    Other,
}

impl Events for ToasterEvents {}

#[derive(Debug, Copy, Clone, PartialEq)]
struct ToastCounter {
    counter: i32,
}

impl Default for ToastCounter {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

impl<'a> Aggregator<ToasterEvents, &'a str, PgQuery> for ToastCounter {
    fn apply_event(acc: Self, event: &ToasterEvents) -> Self {
        let counter = match event {
            ToasterEvents::Inc(inc) => acc.counter + inc.by,
            ToasterEvents::Dec(dec) => acc.counter - dec.by,
            _ => acc.counter,
        };

        Self { counter, ..acc }
    }

    fn query() -> PgQuery {
        PgQuery(String::from(
            "select * from events where data->>'test_field' = $1",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use event_store_rs::Store;

    #[test]
    fn it_aggregates_events() {
        let events = vec![
            ToasterEvents::Inc(Increment { by: 1 }),
            ToasterEvents::Inc(Increment { by: 1 }),
            ToasterEvents::Dec(Decrement { by: 2 }),
            ToasterEvents::Inc(Increment { by: 2 }),
            ToasterEvents::Dec(Decrement { by: 3 }),
            ToasterEvents::Dec(Decrement { by: 3 }),
        ];

        let result: ToastCounter = events
            .iter()
            .fold(ToastCounter::default(), ToastCounter::apply_event);

        assert_eq!(result, ToastCounter { counter: -4 });
    }

    #[test]
    fn thing() {
        let store = PgStore::new();

        let _entity: ToastCounter = store.aggregate("inc_dec");
    }
}
