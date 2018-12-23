// enable the await! macro, async support, and the new std::Futures api.
#![feature(await_macro, async_await, futures_api)]
// only needed if we want to manually write a method to go forward from 0.1 to 0.3 future,
// or manually implement a std future (it provides Pin and Unpin):
#![feature(pin)]
// only needed to manually implement a std future:
#![feature(arbitrary_self_types)]

// Bring tokio's shimmed await!() into scope
// #[macro_use]
// extern crate tokio;

pub mod aggregator;
pub mod amqp;
pub mod event;
pub mod event_context;
pub mod event_handler;
pub mod event_replay;
pub mod pg;
pub mod store;
pub mod store_query;
pub mod subscribable_store;
#[doc(hidden)]
pub mod test_helpers;

use futures::Future as OldFuture;
use std::future::Future as NewFuture;
use std::pin::Unpin;

// converts from a new style Future to an old style one:
// TODO: No pub
pub fn backward<I, E>(
    f: impl NewFuture<Output = Result<I, E>>,
) -> impl OldFuture<Item = I, Error = E> {
    use tokio_async_await::compat::backward;
    backward::Compat::new(f)
}

// converts from an old style Future to a new style one:
// TODO: No pub
pub fn forward<I, E>(
    f: impl OldFuture<Item = I, Error = E> + Unpin,
) -> impl NewFuture<Output = Result<I, E>> {
    use tokio_async_await::compat::forward::IntoAwaitable;
    f.into_awaitable()
}

pub use crate::aggregator::*;
pub use crate::amqp::*;
pub use crate::event::Event;
pub use crate::event_handler::*;
pub use crate::event_replay::*;
pub use crate::pg::*;
pub use crate::store::*;
pub use crate::store_query::*;
pub use crate::subscribable_store::*;
#[doc(hidden)]
pub use crate::test_helpers::*;
