//! Event handler trait

use crate::event::Event;
use crate::store::Store;
use event_store_derive_internals::EventData;

/// Event handler trait
pub trait EventHandler: Sized + EventData {
    /// The method called when an incoming event is received
    fn handle_event(_event: Event<Self>, _saver: &Store) {}
}
