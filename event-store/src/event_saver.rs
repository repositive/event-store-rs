//! Lightweight standin for eventual `StoreAdapter`
//!
//! This only saves events. Eventually any occurrence should be replaced with a proper store adapter

use crate::event::Event;
use crate::pg::pg_save;
use event_store_derive_internals::EventData;
use futures::Future;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use std::io;

#[derive(Debug, Clone)]
pub struct EventSaver {
    pool: Pool<PostgresConnectionManager>,
}

impl EventSaver {
    pub fn new(pool: Pool<PostgresConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn save<ED>(&self, event: &Event<ED>) -> Box<dyn Future<Item = (), Error = io::Error>>
    where
        ED: EventData + 'static,
    {
        Box::new(pg_save(self.pool.get().unwrap(), event))
    }
}
