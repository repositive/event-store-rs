use chrono::prelude::*;
use event_context::EventContext;
use event_store_derive_internals::Events;
use uuid::Uuid;

/// Event with `EventData`, `EventContext` and a `Uuid` ID
///
/// This is what gets stored in the store and emitted from the emitter
// TODO: Make `pub` -> `crate` when it's stabilised
#[derive(Serialize, Deserialize)]
pub struct Event<D> {
    /// Event data payload
    pub data: D,

    /// Event context
    pub context: EventContext,

    /// Event UUID
    pub id: Uuid,
}

impl<D> Event<D>
where
    D: Events,
{
    /// Get the ID of this event
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the data of this event
    pub fn data(&self) -> &D {
        &self.data
    }

    /// Get the context of this event
    pub fn context(&self) -> &EventContext {
        &self.context
    }

    /// Create a new event
    pub fn new(data: D, id: Uuid, context: EventContext) -> Self {
        Self { data, context, id }
    }

    /// Create a new event from some data. `context.time` is set to now, `id` to a new V4 ID
    ///
    /// The rest of the context is left empty
    pub fn from_data(data: D) -> Self {
        Self {
            data,
            id: Uuid::new_v4(),
            context: EventContext {
                action: None,
                subject: None,
                time: Utc::now(),
            },
        }
    }

    /// Create a copied event with the given ID
    ///
    /// ```
    /// # extern crate uuid;
    /// # extern crate event_store;
    /// # use uuid::Uuid;
    /// # use event_store::testhelpers::*;
    /// # use event_store::Event;
    /// # let example_data = TestEvents::Inc(TestIncrementEvent {
    /// #     by: 1,
    /// #     ident: "it_aggregates_events".into(),
    /// # });
    /// #
    /// let event_id = Uuid::new_v4();
    /// let evt = Event::from_data(example_data).with_id(event_id);
    ///
    /// assert_eq!(evt.id(), event_id);
    /// ```
    pub fn with_id(self, id: Uuid) -> Self {
        Self { id, ..self }
    }
}
