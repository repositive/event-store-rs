//! Store adapter backed by Postgres

use adapters::stub::StubQuery;
use adapters::StoreAdapter;
use chrono::{DateTime, Utc};
use futures::future::ok as FutOk;
use utils::BoxedFuture;

use Event;
use EventData;
use Events;

/// Postgres store adapter
#[derive(Clone)]
pub struct StubStoreAdapter {}

impl<'a> StubStoreAdapter {
    /// Create a new StubStore from a Postgres DB connection
    pub fn new() -> Self {
        Self {}
    }
}
impl<'a> StoreAdapter<StubQuery> for StubStoreAdapter {
    fn read<'b, E>(
        &self,
        _query: StubQuery,
        _since: Option<DateTime<Utc>>,
    ) -> BoxedFuture<'b, Vec<E>, String>
    where
        E: Events + Send + 'b,
    {
        Box::new(FutOk(Vec::new()))
    }

    fn save<'b, ED: EventData + Sync + Send + 'b>(
        &self,
        _event: &'b Event<ED>,
    ) -> BoxedFuture<'b, (), String> {
        Box::new(FutOk(()))
    }

    fn last_event<'b, ED: EventData + Send + 'b>(
        &self,
    ) -> BoxedFuture<'b, Option<Event<ED>>, String> {
        Box::new(FutOk(None))
    }
}
