use crate::event::Event;
use crate::store::Store;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;

pub trait EventHandler: Sized + EventData {
    fn handle_event(event: Event<Self>, saver: Store);
}
