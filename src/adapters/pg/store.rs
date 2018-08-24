//! Store adapter backed by Postgres

use super::super::StoreAdapter;
use chrono::prelude::*;
use postgres::types::ToSql;
use postgres::Connection;
use serde::Deserialize;
use std::marker::PhantomData;
use Events;
use StoreQuery;

/// Representation of a Postgres query and args
pub struct PgQuery<'a> {
    /// Query string with placeholders
    query: &'a str,

    /// Arguments to use for the query
    args: Vec<Box<ToSql>>,
}

impl<'a> StoreQuery for PgQuery<'a> {}

impl<'a> PgQuery<'a> {
    /// Create a new query from a query string and arguments
    pub fn new(query: &'a str, args: Vec<Box<ToSql>>) -> Self {
        Self { query, args }
    }
}

// pub struct PgStoreAdapter {}

// impl PgStoreAdapter {
//     /// Create a new PgStore from a Postgres DB connection
//     pub fn new(conn: Connection) -> Self {
//         Self {
//             phantom: PhantomData,
//             conn,
//         }
//     }
// }

// impl<'a, E> StoreAdapter<E, PgQuery<'a>> for PgStoreAdapter
// where
//     E: Events + Deserialize<'a>,
// {
//     /// Read a list of events matching a query
//     fn read(&self, query: Q, since: Option<DateTime<Utc>>) -> Result<Vec<Box<E>>, String> {}

//     /// Save an event to the store
//     fn save(&self, event: E) -> Result<(), String> {}
// }
