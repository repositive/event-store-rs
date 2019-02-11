mod pg;

pub use self::pg::{PgQuery, PgStoreAdapter, SaveResult, SaveStatus};
use chrono::prelude::*;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LastHandledEvent {
    domain: String,
    event_namespace: String,
    event_type: String,
    event_id: Uuid,
    time: DateTime<Utc>,
    sequence_number: i64,
}
