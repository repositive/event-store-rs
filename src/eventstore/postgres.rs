use event::Event;
use eventstore::EventStore;

/// Postgres-backed implementation of an event store
pub struct PostgresEventStore;

impl<E> EventStore<E> for PostgresEventStore
where
    E: Event,
{
    fn save(&mut self, _event: &E) -> Result<(), String>
    where
        E: Event,
    {
        Ok(())
    }
}
