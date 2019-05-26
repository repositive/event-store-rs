use chrono::{DateTime, Utc};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Clone)]
pub struct Event<D> {
    pub id: Uuid,
    pub sequence_number: Option<u64>,
    pub entity_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub data: Option<D>,
    pub context: Context,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Clone)]
pub struct Context {
    pub subject_id: Uuid,
    pub hostname: String,
    pub username: String,
    pub ip: Option<IpAddr>,
    pub purge: Option<Purge>,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug, Clone)]
pub struct Purge {
    pub purged_at: DateTime<Utc>,
    pub hostname: String,
    pub username: String,
    pub ip: Option<IpAddr>,
}
