use futures::future::Future;

pub type BoxedFuture<T, E> = Box<Future<Item = T, Error = E> + Send>;
