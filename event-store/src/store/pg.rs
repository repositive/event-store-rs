use chrono::{DateTime, Utc};
use crate::Event;
use crate::TestEvents;
use fallible_iterator::FallibleIterator;
use postgres::error::DUPLICATE_COLUMN;
use postgres::types::ToSql;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use serde_json::{from_value, to_value, Value as JsonValue};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug)]
pub struct PgQuery {
    pub query: String,
    pub args: Vec<Box<ToSql>>,
}

impl PgQuery {
    pub fn new(query: &str, args: Vec<Box<ToSql>>) -> Self {
        Self {
            query: query.into(),
            args,
        }
    }

    pub fn unique_id(&self) -> String {
        let hash = Sha256::digest(format!("{:?}:[{}]", self.args, self.query).as_bytes());

        hash.iter().fold(String::new(), |mut acc, hex| {
            acc.push_str(&format!("{:X}", hex));
            acc
        })
    }
}

#[derive(Debug, Clone)]
pub struct PgStoreAdapter {
    pool: Pool<PostgresConnectionManager>,
}

impl PgStoreAdapter {
    pub fn new(pool: Pool<PostgresConnectionManager>) -> Self {
        Self { pool }
    }

    fn generate_query(initial_query: &PgQuery, since: Option<DateTime<Utc>>) -> String {
        if let Some(timestamp) = since {
            String::from(format!(
            "SELECT * FROM ({}) AS events WHERE events.context->>'time' >= '{}' ORDER BY events.context->>'time' ASC",
            initial_query.query, timestamp,
        ))
        } else {
            String::from(format!(
                "SELECT * FROM ({}) AS events ORDER BY events.context->>'time' ASC",
                initial_query.query
            ))
        }
    }

    pub fn read(
        &self,
        query: PgQuery,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<Event<TestEvents>>, String> {
        Ok(Vec::new())
    }

    pub fn save(&self, event: &Event<TestEvents>) -> Result<(), String> {
        Ok(())
    }

    pub fn last_event(&self) -> Result<Option<Event<TestEvents>>, String> {
        Ok(None)
    }
}
