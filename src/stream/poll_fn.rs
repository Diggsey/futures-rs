//! Definition of the `PollFn` combinator

use std::marker::PhantomData;

use {Stream, Poll, PollableStream};

/// A stream which adapts a function returning `Poll`.
///
/// Created by the `poll_fn` function.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct PollFn<F, TaskT> {
    inner: F,
    phantom: PhantomData<FnMut(TaskT)>
}

/// Creates a new stream wrapping around a function returning `Poll`.
///
/// Polling the returned stream delegates to the wrapped function.
///
/// # Examples
///
/// ```
/// use futures::stream::poll_fn;
/// use futures::{Async, Poll};
///
/// let mut counter = 1usize;
///
/// let read_stream = poll_fn(move || -> Poll<Option<String>, std::io::Error> {
///     if counter == 0 { return Ok(Async::Ready(None)); }
///     counter -= 1;
///     Ok(Async::Ready(Some("Hello, World!".to_owned())))
/// });
/// ```
pub fn poll_fn<T, E, F, TaskT>(f: F) -> PollFn<F, TaskT>
where
    F: FnMut(&mut TaskT) -> Poll<Option<T>, E>,
{
    PollFn { inner: f, phantom: PhantomData }
}

impl<T, E, F, TaskT> Stream for PollFn<F, TaskT>
where
    F: FnMut(&mut TaskT) -> Poll<Option<T>, E>,
{
    type Item = T;
    type Error = E;
}

impl<T, E, F, TaskT> PollableStream<TaskT> for PollFn<F, TaskT>
where
    F: FnMut(&mut TaskT) -> Poll<Option<T>, E>,
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<T>, E> {
        (self.inner)(task)
    }
}
