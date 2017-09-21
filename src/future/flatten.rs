use {Future, IntoFuture, Poll, Pollable};
use core::fmt;
use super::chain::Chain;

/// Future for the `flatten` combinator, flattening a future-of-a-future to get just
/// the result of the final future.
///
/// This is created by the `Future::flatten` method.
#[must_use = "futures do nothing unless polled"]
pub struct Flatten<A> where A: Future, A::Item: IntoFuture {
    state: Chain<A, <A::Item as IntoFuture>::Future, ()>,
}

impl<A> fmt::Debug for Flatten<A>
    where A: Future + fmt::Debug,
          A::Item: IntoFuture,
          <<A as IntoFuture>::Item as IntoFuture>::Future: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Flatten")
            .field("state", &self.state)
            .finish()
    }
}

pub fn new<A>(future: A) -> Flatten<A>
    where A: Future,
          A::Item: IntoFuture,
{
    Flatten {
        state: Chain::new(future, ()),
    }
}

impl<A> Future for Flatten<A>
    where A: Future,
          A::Item: IntoFuture,
          <A::Item as IntoFuture>::Error: From<A::Error>
{
    type Item = <<A as Future>::Item as IntoFuture>::Item;
    type Error = <<A as Future>::Item as IntoFuture>::Error;
}

impl<A, TaskT> Pollable<TaskT> for Flatten<A>
    where A: Pollable<TaskT>,
          A::Item: IntoFuture,
          <A::Item as IntoFuture>::Future: Pollable<TaskT>,
          <A::Item as IntoFuture>::Error: From<A::Error>
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<Self::Item, Self::Error> {
        self.state.poll(|a, ()| {
            let future = a?.into_future();
            Ok(Err(future))
        }, task)
    }
}
