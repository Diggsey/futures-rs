//! Definition of the `PollFn` adapter combinator

use std::marker::PhantomData;

use {Future, Poll, Pollable};

/// A future which adapts a function returning `Poll`.
///
/// Created by the `poll_fn` function.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct PollFn<F, TaskT> {
    inner: F,
    phantom: PhantomData<FnMut(TaskT)>
}

/// Creates a new future wrapping around a function returning `Poll`.
///
/// Polling the returned future delegates to the wrapped function.
///
/// # Examples
///
/// ```
/// use futures::future::poll_fn;
/// use futures::{Async, Poll};
///
/// fn read_line() -> Poll<String, std::io::Error> {
///     Ok(Async::Ready("Hello, World!".into()))
/// }
///
/// let read_future = poll_fn(read_line);
/// ```
pub fn poll_fn<T, E, F, TaskT>(f: F) -> PollFn<F, TaskT>
    where F: FnMut(&mut TaskT) -> ::Poll<T, E>
{
    PollFn { inner: f, phantom: PhantomData }
}

impl<T, E, F, TaskT> Future for PollFn<F, TaskT>
    where F: FnMut(&mut TaskT) -> Poll<T, E>
{
    type Item = T;
    type Error = E;
}

impl<T, E, F, TaskT> Pollable<TaskT> for PollFn<F, TaskT>
    where F: FnMut(&mut TaskT) -> Poll<T, E>
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<T, E> {
        (self.inner)(task)
    }
}
