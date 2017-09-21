use std::prelude::v1::*;
use std::any::Any;
use std::panic::{catch_unwind, UnwindSafe, AssertUnwindSafe};

use {Future, Pollable, Poll, Async};

/// Future for the `catch_unwind` combinator.
///
/// This is created by the `Future::catch_unwind` method.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct CatchUnwind<F> where F: Future {
    future: Option<F>,
}

pub fn new<F>(future: F) -> CatchUnwind<F>
    where F: Future + UnwindSafe,
{
    CatchUnwind {
        future: Some(future),
    }
}

impl<F> Future for CatchUnwind<F>
    where F: Future + UnwindSafe,
{
    type Item = Result<F::Item, F::Error>;
    type Error = Box<Any + Send>;

}

impl<F, TaskT> Pollable<TaskT> for CatchUnwind<F>
    where F: Pollable<TaskT> + UnwindSafe, TaskT: UnwindSafe
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<Self::Item, Self::Error> {
        let mut future = self.future.take().expect("cannot poll twice");
        let (res, future) = catch_unwind(AssertUnwindSafe(|| (future.poll(task), future)))?;
        match res {
            Ok(Async::NotReady) => {
                self.future = Some(future);
                Ok(Async::NotReady)
            }
            Ok(Async::Ready(t)) => Ok(Async::Ready(Ok(t))),
            Err(e) => Ok(Async::Ready(Err(e))),
        }
    }
}

impl<F: Future> Future for AssertUnwindSafe<F> {
    type Item = F::Item;
    type Error = F::Error;
}

impl<F, TaskT> Pollable<TaskT> for AssertUnwindSafe<F>
    where F: Pollable<TaskT>
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<F::Item, F::Error> {
        self.0.poll(task)
    }
}
