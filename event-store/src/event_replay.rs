use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::store::Store;
use chrono::prelude::*;
use event_store_derive::*;
use log::debug;
use serde_derive::*;

#[derive(EventData, Debug)]
#[event_store(namespace = "_eventstore")]
pub struct EventReplayRequested {
    requested_event_namespace: String,
    requested_event_type: String,
    since: DateTime<Utc>,
}

impl EventHandler for EventReplayRequested {
    fn handle_event(event: Event<Self>, _store: &Store) {
        debug!("Event replay received {:?}", event);

        // TODO: Implement replay logic
    }
}
