use crate::event_context::EventContext;
use chrono::prelude::*;
use event_store_derive_internals::EventData;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

/// Event with `EventData`, `EventContext` and a `Uuid` ID
///
/// This is what gets stored in the store and emitted from the emitter
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    D: EventData,
{
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
    pub fn with_id(self, id: Uuid) -> Self {
        Self { id, ..self }
    }
}
