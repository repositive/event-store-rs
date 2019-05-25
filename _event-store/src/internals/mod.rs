pub mod test_helpers;

use futures::Future as OldFuture;
use std::future::Future as NewFuture;

// converts from a new style Future to an old style one
pub fn backward<I, E>(
    f: impl NewFuture<Output = Result<I, E>>,
) -> impl OldFuture<Item = I, Error = E> {
    use tokio_async_await::compat::backward;
    backward::Compat::new(f)
}

// converts from an old style Future to a new style one
pub fn forward<I, E>(
    f: impl OldFuture<Item = I, Error = E> + Unpin,
) -> impl NewFuture<Output = Result<I, E>> {
    use tokio_async_await::compat::forward::IntoAwaitable;
    f.into_awaitable()
}
