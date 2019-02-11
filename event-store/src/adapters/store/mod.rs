mod pg;

pub use self::pg::{PgQuery, PgStoreAdapter, SaveResult, SaveStatus};
use chrono::prelude::*;
use uuid::Uuid;

/// Last handled event metadata
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LastHandledEvent {
    /// The domain this event was received by
    pub domain: String,

    /// The event's namespace (emitting domain)
    pub event_namespace: String,

    /// Event type
    pub event_type: String,

    /// Event ID
    pub event_id: Uuid,

    /// When the event was created
    pub time: DateTime<Utc>,

    /// Sequence number of the event
    pub sequence_number: i64,
}
