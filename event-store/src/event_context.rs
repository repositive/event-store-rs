use chrono::prelude::*;
use serde_json::Value as JsonValue;

/// Event context
///
/// Contains metadata for event and, most importantly, the creation time
#[derive(Serialize, Deserialize, Debug)]
pub struct EventContext {
    /// TODO: What is this?
    pub action: Option<String>,

    /// Optional event "subject" or metadata
    pub subject: Option<JsonValue>,

    /// Event creation time
    pub time: DateTime<Utc>,
}
