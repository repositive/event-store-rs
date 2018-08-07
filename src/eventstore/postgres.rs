use event::Event;
use eventstore::EventStore;

/// Postgres-backed implementation of an event store
pub struct PostgresEventStore;

impl EventStore for PostgresEventStore {
    fn save<E>(&mut self, _event: &E) -> Result<(), String>
    where
        E: Event,
    {
        Ok(())
    }
}
