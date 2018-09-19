#[test]
fn create_store() {
    let store_adapter = StubStoreAdapter::new();
    let cache_adapter = StubCacheAdapter::new();
    let emitter_adapter = StubEmitterAdapter::new();

    EventStore::new(store_adapter, cache_adapter, emitter_adapter)
}
