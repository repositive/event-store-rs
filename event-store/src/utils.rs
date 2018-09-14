use futures::future::Future;
use futures::stream::Stream;
use std::sync::Arc;

pub type BoxedFuture<'a, T, E> = Box<Future<Item = T, Error = E> + Send + Sync + 'a>;
pub type ArcFuture<'a, T, E> = Arc<Future<Item = T, Error = E> + 'a>;

pub type BoxedStream<'a, T, E> = Box<Stream<Item = T, Error = E> + Send + Sync + 'a>;
pub type ArcStream<'a, T, E> = Arc<Stream<Item = T, Error = E> + 'a>;
