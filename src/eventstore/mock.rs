use event::Event;
use eventstore::EventStore;

/// Mock implementation of an event store for testing purposes
pub struct MockEventStore<E: Event> {
    events: Vec<Box<E>>,
}

impl<E> MockEventStore<E>
where
    E: Event,
{
    pub fn new() -> Self {
        let events: Vec<Box<E>> = Vec::new();

        Self { events }
    }
}

impl<E> EventStore<E> for MockEventStore<E>
where
    E: Event,
{
    fn save(&mut self, event: &E) -> Result<(), String> {
        self.events.push(Box::new(*event));

        Ok(())
    }
}
