/// A query to be passed to the store
///
/// This trait must be implemented for whichever type you want to pass to a particular store. See
/// impls below for examples.
pub trait StoreQuery<'a, Q, A> {
    /// Must return a query for the specific store
    fn get_query(&self) -> Q;
}
