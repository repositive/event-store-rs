use futures::future::Future;

pub type BoxedFuture<'a, T, E> = Box<Future<Item = T, Error = E> + Send + 'a>;
