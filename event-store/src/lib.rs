#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate event_store_derive;
#[macro_use]
extern crate log;

pub mod aggregator;
pub mod amqp;
pub mod event;
pub mod event_context;
pub mod event_saver;
pub mod pg;
pub mod store_query;
#[doc(hidden)]
pub mod test_helpers;

use event_store_derive_internals::Events;
use futures::prelude::*;
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use std::io;

pub use crate::aggregator::*;
pub use crate::amqp::*;
pub use crate::event::Event;
pub use crate::event_saver::*;
pub use crate::pg::*;
pub use crate::store_query::*;
#[doc(hidden)]
pub use crate::test_helpers::*;

pub fn store_aggregate<E, T, QA>(
    conn: PooledConnection<PostgresConnectionManager>,
    query_args: QA,
) -> impl Future<Item = T, Error = io::Error>
where
    E: Events,
    T: Aggregator<E, QA, PgQuery>,
    QA: Clone,
{
    let store_query = T::query(query_args.clone());
    let cache_key = store_query.unique_id();

    pg_cache_read(conn, cache_key)
        .and_then(
            |(conn, cache_result): (
                PooledConnection<PostgresConnectionManager>,
                Option<CacheResult<T>>,
            )| {
                let (initial_state, since) = cache_result
                    .map(|res| (res.0, Some(res.1)))
                    .unwrap_or_else(|| (T::default(), None));

                pg_read(conn, store_query, since).map(|events: Vec<E>| (events, initial_state))
            },
        )
        .map(|(events, initial_state)| events.iter().fold(initial_state, T::apply_event))
}
