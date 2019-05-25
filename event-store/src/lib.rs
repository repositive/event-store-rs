mod event;
mod event_store;

pub use crate::event::Event;
pub use crate::event_store::EventStore;
pub use event_store_derive::{CreateEvents, UpdateEvents};
pub use event_store_derive_internals::{EventStoreCreateEvents, EventStoreUpdateEvents};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
