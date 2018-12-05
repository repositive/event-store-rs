/// A query to be passed to the store
///
/// This trait must be implemented for whichever type you want to pass to a particular store. See
/// impls below for examples.
pub trait StoreQuery {
    /// You must return a unique identifier based on the query you are performing. This identifier
    /// will then be used to identify the cache and optimize the aggregations using memoization
    fn unique_id(&self) -> String;
}
