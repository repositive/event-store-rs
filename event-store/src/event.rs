use chrono::{DateTime, Utc};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Clone)]
pub struct Event<D, ET> {
    pub id: Uuid,
    pub sequence_number: u64,
    // event_namespace: String,
    // event_type: String,
    pub entity_id: Uuid,
    pub entity_type: ET,
    pub created_at: DateTime<Utc>,
    pub data: Option<D>,
    pub context: Context,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Clone)]
pub struct Context {
    subject_id: Uuid,
    hostname: String,
    username: String,
    ip: Option<IpAddr>,
    purge: Option<Purge>,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Clone)]
pub struct Purge {
    purged_at: DateTime<Utc>,
    hostname: String,
    username: String,
    ip: Option<IpAddr>,
}
