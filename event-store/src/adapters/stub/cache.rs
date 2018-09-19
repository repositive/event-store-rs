//! Cache adapter backed by postgres

use adapters::{CacheAdapter, CacheResult};
use futures::future::ok as FutOk;
use serde::{de::DeserializeOwned, Serialize};
use utils::BoxedFuture;

/// Postgres cache adapter
#[derive(Clone)]
pub struct StubCacheAdapter {}

impl StubCacheAdapter {
    /// Create a new StubStore from a Postgres DB connection
    pub fn new() -> Self {
        Self {}
    }
}

impl CacheAdapter for StubCacheAdapter {
    fn set<'a, V: Serialize + Send + 'a>(
        &self,
        _key: String,
        _value: V,
    ) -> BoxedFuture<'a, (), String> {
        Box::new(FutOk(()))
    }

    fn get<'a, T>(&self, _key: String) -> BoxedFuture<'a, Option<CacheResult<T>>, String>
    where
        T: DeserializeOwned + Send + 'a,
    {
        Box::new(FutOk(None))
    }
}
