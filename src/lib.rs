//! Event store module for event-driven applications

#![deny(missing_docs)]

mod event;
mod eventstore;

pub use event::Event;
pub use eventstore::{EventStore, PostgresEventStore};

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEvent {}

    impl Event for TestEvent {}

    #[test]
    fn it_has_a_postgres_store() {
        let mut store = PostgresEventStore {};
        let e = TestEvent {};

        assert_eq!(store.save(&e), Ok(()));
    }
}
