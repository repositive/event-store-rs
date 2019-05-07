use chrono::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Event data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventData {
    pub event_namespace: String,
    pub event_type: String,

    #[serde(flatten)]
    pub payload: serde_json::Value,

    /// Legacy combined `type` field. Removed when saving into destination DB
    #[serde(skip_serializing)]
    #[serde(rename = "type")]
    legacy_type: String,
}

/// Event context
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventContext {
    pub action: Option<String>,

    /// Event "subject" or metadata
    #[serde(default)]
    pub subject: HashMap<String, serde_json::Value>,

    /// Event creation time
    pub time: DateTime<Utc>,
}

/// An event
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub id: Uuid,
    pub data: EventData,
    pub context: EventContext,
}
