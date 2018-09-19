//! Cache adapter backed by postgres

use adapters::{CacheAdapter, CacheResult};
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
        key: String,
        value: V,
    ) -> BoxedFuture<'a, (), String> {

    }

    fn get<'a, T>(&self, key: String) -> BoxedFuture<'a, Option<CacheResult<T>>, String>
    where
        T: DeserializeOwned + Send + 'a,
    {

    }
}
