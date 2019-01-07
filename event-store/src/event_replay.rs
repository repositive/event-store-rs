use event_store_derive_internals::Events;
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

async fn test() -> Result<u32, String> {
    debug!("TEST FN");
    Ok(100)
}

impl EventHandler for EventReplayRequested {
    fn handle_event(event: Event<Self>, _store: Store){
        debug!("HANDLER");
        debug!("Event replay received {:?}", event);

        // TODO: Implement replay logic

        let store = _store.clone();

        tokio::spawn_async(
            async {
                await!(test()).unwrap();

                let since = event.context.time;
                let ns = event.data.requested_event_namespace;
                let ty = event.data.requested_event_type;

                let _events = await!(store.read_events_since::<Self>(&ns, &ty, since)).expect("Could not read events to replay");
            },
        );
    }
}
