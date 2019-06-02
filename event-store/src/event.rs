use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Event definition
#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Clone)]
pub struct Event<D> {
    /// Event ID
    pub id: Uuid,

    /// Sequence number
    ///
    /// This field is autogenerated by the database and should not be set in user code
    pub sequence_number: Option<u64>,

    /// The ID of the entity (user, organisation, etc) that this event aggregates into
    pub entity_id: Uuid,

    /// The ID of the creator of this event
    pub subject_id: Uuid,

    /// Purger subject ID
    ///
    /// Will be `None` if event is not purged
    pub purger_id: Option<Uuid>,

    /// Event data
    pub data: Option<D>,

    /// The time at which this event was created
    pub created_at: DateTime<Utc>,

    /// The time at which this event was purged, if any
    pub purged_at: Option<DateTime<Utc>>,
}
