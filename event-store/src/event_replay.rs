use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::store::Store;
use chrono::prelude::*;
use event_store_derive::*;
use event_store_derive_internals::EventData;
use log::debug;
use serde_derive::*;

#[derive(EventData, Debug)]
#[event_store(namespace = "_eventstore")]
pub struct EventReplayRequested {
    requested_event_namespace: String,
    requested_event_type: String,
    since: DateTime<Utc>,
}

impl EventReplayRequested {
    pub fn from_event<ED>(since: DateTime<Utc>) -> Event<Self>
    where
        ED: EventData,
    {
        Event::from_data(Self {
            requested_event_namespace: ED::event_namespace().to_string(),
            requested_event_type: ED::event_type().to_string(),
            since,
        })
    }
}

impl EventHandler for EventReplayRequested {
    fn handle_event(event: Event<Self>, store: &Store) {
        debug!("Event replay received {:?}", event);

        let store = store.clone();

        tokio::spawn_async(
            async move {
                let since = event.data.since;
                let ns = event.data.requested_event_namespace;
                let ty = event.data.requested_event_type;

                let events = await!(store.read_events_since(&ns, &ty, since))
                    .expect("Could not read events to replay");

                debug!("Found {} events to replay", events.len());

                for event in events {
                    debug!("Replay event {}", event["id"]);

                    await!(store.emit_value(&ns, &ty, &event)).unwrap();
                }
            },
        );
    }
}
