pub mod aggregator;
pub mod amqp;
pub mod event;
pub mod event_context;
pub mod event_handler;
pub mod event_saver;
pub mod pg;
pub mod store_query;
#[doc(hidden)]
pub mod test_helpers;

use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use futures::future;
use futures::prelude::*;
use lapin_futures::channel::Channel;
use log::{debug, trace};
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use std::io;
use tokio::net::TcpStream;
use tokio_core::reactor::Handle;

pub use crate::aggregator::*;
pub use crate::amqp::*;
pub use crate::event::Event;
pub use crate::event_handler::*;
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

                trace!(
                    "Aggregate initial state {:?}, since {:?}",
                    initial_state,
                    since
                );

                pg_read(conn, store_query, since).map(|events: Vec<E>| (events, initial_state))
            },
        )
        .map(|(events, initial_state)| {
            trace!("Read {} events to aggregate", events.len());

            events.iter().fold(initial_state, T::apply_event)
        })
}

pub fn store_subscribe<ED>(
    amqp_channel: Channel<TcpStream>,
    saver: EventSaver,
    handle: Handle,
) -> impl Future<Item = (), Error = io::Error>
where
    ED: EventData + EventHandler + 'static,
{
    debug!("Create subscription");

    amqp_create_consumer(
        amqp_channel.clone(),
        "rando_queue".into(),
        "test_exchange".into(),
        move |ev: Event<ED>| {
            let fut = saver
                .save(ev)
                .and_then(|ev| {
                    trace!("Handle event ID {}", ev.id);

                    ED::handle_event(ev);

                    future::ok(())
                })
                .map_err(|_| ());

            handle.spawn(fut);
        },
    )
}

pub fn store_save<ED>(
    saver: EventSaver,
    amqp_channel: Channel<TcpStream>,
    event: Event<ED>,
) -> impl Future<Item = (), Error = io::Error>
where
    ED: EventData,
{
    // TODO: Domain namespace for queue name

    saver.save(event).and_then(|event| {
        amqp_emit_event(
            amqp_channel,
            // TODO: Dynamic queue name with domain namespace
            "rando_queue".into(),
            // TODO: Configurable string
            "test_exchange".into(),
            &event,
        )
    })
}
