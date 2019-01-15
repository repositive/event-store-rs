//! Aggregator trait

use crate::store_query::StoreQuery;
use event_store_derive_internals::Events;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// TODO: Port docs from `_event-store/src/aggregator`
/// Aggregator trait
pub trait Aggregator<E: Events, A: Clone, Q: StoreQuery>:
    Clone + Debug + Default + PartialEq + Serialize + for<'de> Deserialize<'de>
{
    /// Apply an event `E` to `acc`, returning a copy of `Self` with updated fields. Can also just
    /// return `acc` if nothing has changed.
    fn apply_event(acc: Self, event: &E) -> Self;

    /// Produce a query object from some query arguments
    fn query(query_args: A) -> Q;
}
