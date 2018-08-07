mod postgres;

use event::Event;

pub use self::postgres::PostgresEventStore;

/// Event store trait
///
/// All event stores must implement this trait
pub trait EventStore {
    /// Save an event to the store
    fn save<E>(&mut self, event: &E) -> Result<(), String>
    where
        E: Event;
}
