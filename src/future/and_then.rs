use {Future, IntoFuture, Poll, Pollable};
use super::chain::Chain;

/// Future for the `and_then` combinator, chaining a computation onto the end of
/// another future which completes successfully.
///
/// This is created by the `Future::and_then` method.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct AndThen<A, B, F> where A: Future, B: IntoFuture {
    state: Chain<A, B::Future, F>,
}

pub fn new<A, B, F>(future: A, f: F) -> AndThen<A, B, F>
    where A: Future,
          B: IntoFuture,
{
    AndThen {
        state: Chain::new(future, f),
    }
}

impl<A, B, F> Future for AndThen<A, B, F>
    where A: Future,
          B: IntoFuture<Error=A::Error>,
          F: FnOnce(A::Item) -> B,
{
    type Item = B::Item;
    type Error = B::Error;
}

impl<A, B, F, TaskT> Pollable<TaskT> for AndThen<A, B, F>
    where A: Future + Pollable<TaskT>,
          B: IntoFuture<Error=A::Error>,
          F: FnOnce(A::Item) -> B,
          <B as IntoFuture>::Future: Pollable<TaskT>
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<B::Item, B::Error> {
        self.state.poll(|result, f| {
            result.map(|e| {
                Err(f(e).into_future())
            })
        }, task)
    }
}
