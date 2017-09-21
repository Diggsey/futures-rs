//! Definition of the `Option` (optional step) combinator

use {Future, Poll, Async, Pollable};

impl<F, T, E> Future for Option<F> where F: Future<Item=T, Error=E> {
    type Item = Option<T>;
    type Error = E;
}

impl<F, T, E, TaskT> Pollable<TaskT> for Option<F> where F: Pollable<TaskT, Item=T, Error=E> {
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<T>, E> {
        match *self {
            None => Ok(Async::Ready(None)),
            Some(ref mut x) => x.poll(task).map(|x| x.map(Some)),
        }
    }
}
