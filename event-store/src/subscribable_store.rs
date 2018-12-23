use crate::aggregator::Aggregator;
use crate::amqp::*;
use crate::event::Event;
use crate::event_handler::EventHandler;
use crate::event_replay::EventReplayRequested;
use crate::pg::PgQuery;
use crate::store::Store;
use chrono::naive::NaiveDateTime;
use chrono::prelude::*;
use event_store_derive_internals::EventData;
use event_store_derive_internals::Events;
use lapin_futures::channel::Channel;
use log::{debug, trace};
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use std::fmt;
use std::fmt::Debug;
use std::io;
use std::net::SocketAddr;
use tokio::net::tcp::TcpStream;

#[derive(Clone)]
pub struct SubscribableStore {
    store_namespace: String,
    channel: Channel<TcpStream>,
    inner_store: Store,
}

impl Debug for SubscribableStore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SubscribableStore namespace {}", self.store_namespace)
    }
}

impl SubscribableStore {
    pub async fn new(
        store_namespace: String,
        pool: Pool<PostgresConnectionManager>,
    ) -> Result<Self, io::Error> {
        let addr: SocketAddr = "127.0.0.1:5673".parse().unwrap();

        // TODO: Pass in an AMQP adapter inside a promise instead of doing this here
        let channel = await!(amqp_connect(addr, "test_exchange".into()))?;

        let store = Self {
            channel: channel.clone(),
            store_namespace: store_namespace.clone(),
            inner_store: Store::new(store_namespace, pool, channel),
        };

        await!(store.subscribe_no_replay::<EventReplayRequested>())?;

        Ok(store)

        // amqp_connect(addr, "test_exchange".into())
        //     .map(|channel| )
        //     .and_then(|store| {
        //         debug!("Begin listening for event replay requests");

        //     })
        //     // FIXME: Remove this delay
        //     .and_then(|store| {
        //         // Give the replay consumer some time to settle
        //         Delay::new(Instant::now() + Duration::from_millis(100))
        //             .map_err(|_| io::Error::new(io::ErrorKind::Other, "wait error"))
        //             .map(|_| store)
        //     })
    }

    pub async fn aggregate<'a, T, QA, E>(&'a self, query_args: &'a QA) -> Result<T, io::Error>
    where
        E: Events,
        T: Aggregator<E, QA, PgQuery>,
        QA: Clone + Debug + 'a,
    {
        let res: T = await!(self.inner_store.aggregate::<'a, T, QA, E>(&query_args))?;

        Ok(res)
    }

    pub async fn save<'a, ED>(&'a self, event: &'a Event<ED>) -> Result<(), io::Error>
    where
        ED: EventData + Debug,
    {
        await!(self.inner_store.save(event))
    }

    pub async fn save_no_emit<'a, ED>(&'a self, event: &'a Event<ED>) -> Result<(), io::Error>
    where
        ED: EventData + Debug,
    {
        await!(self.inner_store.save_no_emit(event))
    }

    async fn subscribe_no_replay<ED>(&self) -> Result<(), io::Error>
    where
        ED: EventHandler + Debug + Send + 'static,
    {
        let queue_name = self.namespaced_event_queue_name::<ED>();
        let inner_store = self.inner_store.clone();

        debug!("Begin listening for events on queue {}", queue_name);

        let consumer = amqp_create_consumer(
            self.channel.clone(),
            queue_name,
            "test_exchange".into(),
            move |event: Event<ED>| {
                // TODO: Save event in this closure somewhere

                ED::handle_event(event, &inner_store);
            },
        );

        tokio::spawn_async(consumer);

        Ok(())
    }

    pub async fn subscribe<ED>(&self) -> Result<(), io::Error>
    where
        ED: EventHandler + Debug + Send + 'static,
    {
        let replay_queue_name = self.event_queue_name::<EventReplayRequested>();
        let inner_channel = self.channel.clone();

        await!(self.subscribe_no_replay::<ED>())?;

        let since = await!(self.inner_store.last_event::<ED>())
            .map(|last_event| {
                trace!("Fetched last event {:?}", last_event);

                last_event.map(|ev| ev.context.time).unwrap_or_else(|| {
                    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
                })
            })
            .unwrap_or_else(|_| {
                DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
            });

        let replay_event = EventReplayRequested::from_event::<ED>(since);

        await!(amqp_emit_event(
            inner_channel,
            replay_queue_name,
            "test_exchange".into(),
            &replay_event,
        ))?;

        // .and_then(move |since| {
        //     trace!("Emit replay request for events since {:?}", since);

        //     amqp_emit_event(
        //         inner_channel,
        //         replay_queue_name,
        //         "test_exchange".into(),
        //         &EventReplayRequested::from_event::<ED>(since),
        //     )
        // })
        // // FIXME: Remove this delay
        // .and_then(|_| {
        //     // Give the consumer some time to settle
        //     Delay::new(Instant::now() + Duration::from_millis(100))
        //         .map_err(|_| io::Error::new(io::ErrorKind::Other, "wait error"))
        //         .map(|_| ())
        // })

        Ok(())
    }

    fn namespaced_event_queue_name<ED>(&self) -> String
    where
        ED: EventData,
    {
        format!("{}-{}", self.store_namespace, self.event_queue_name::<ED>())
    }

    fn event_queue_name<ED>(&self) -> String
    where
        ED: EventData,
    {
        format!("{}.{}", ED::event_namespace(), ED::event_type())
    }
}
