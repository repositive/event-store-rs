use event::Event;
use eventstore::EventStore;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;

/// Postgres-backed implementation of an event store
pub struct PostgresEventStore {
    pool: Pool<PostgresConnectionManager>,
}

impl PostgresEventStore {
    /// Create a new Postgres-backed store
    pub fn new(pool: Pool<PostgresConnectionManager>) -> Self {
        Self { pool }
    }
}

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
