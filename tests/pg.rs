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
    Inc(Increment),
    Dec(Decrement),
}

impl Events for ToasterEvents {}

#[derive(Debug, Copy, Clone, PartialEq)]
struct SomeDomainEntity {
    counter: i32,
}

impl Default for SomeDomainEntity {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

impl Aggregator<ToasterEvents, (String, String), PgQuery> for SomeDomainEntity {
    fn apply_event(acc: Self, event: &ToasterEvents) -> Self {
        let counter = match event {
            ToasterEvents::Inc(inc) => acc.counter + inc.by,
            ToasterEvents::Dec(dec) => acc.counter - dec.by,
        };

        Self { counter, ..acc }
    }

    fn query() -> PgQuery {
        PgQuery(String::from("SELECT * FROM events WHERE toast = 'yes'"))
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

        let result: SomeDomainEntity = events
            .iter()
            .fold(SomeDomainEntity::default(), SomeDomainEntity::apply_event);

        assert_eq!(result, SomeDomainEntity { counter: -4 });
    }

    #[test]
    fn thing() {
        let store = PgStore::new();

        let _entity: SomeDomainEntity = store.aggregate(("id".into(), "oenebtio".into()));
    }
}
