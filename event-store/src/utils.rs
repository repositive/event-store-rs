use futures::future::Future;
use futures::stream::Stream;

pub type BoxedFuture<'a, T, E> = Box<Future<Item = T, Error = E> + 'a>;

pub type BoxedStream<'a, T, E> = Box<Stream<Item = T, Error = E> + 'a>;
