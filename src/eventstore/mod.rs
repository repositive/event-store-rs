#[cfg(test)]
mod mock;
mod postgres;

use event::Event;

#[cfg(test)]
pub use self::mock::MockEventStore;
pub use self::postgres::PostgresEventStore;

/// Event store trait
///
/// All event stores must implement this trait
pub trait EventStore<E: Event> {
    /// Save an event to the store
    fn save(&mut self, event: &E) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone)]
    struct TestEvent;

    impl Event for TestEvent {}

    #[test]
    fn it_stores_events() {
        let mut store = MockEventStore::new();

        assert_eq!(store.save(&TestEvent), Ok(()));
    }
}
