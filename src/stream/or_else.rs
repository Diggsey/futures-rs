use {IntoFuture, Future, Poll, Async, PollableStream, Pollable};
use stream::Stream;

/// A stream combinator which chains a computation onto errors produced by a
/// stream.
///
/// This structure is produced by the `Stream::or_else` method.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct OrElse<S, F, U>
    where U: IntoFuture,
{
    stream: S,
    future: Option<U::Future>,
    f: F,
}

pub fn new<S, F, U>(s: S, f: F) -> OrElse<S, F, U>
    where S: Stream,
          F: FnMut(S::Error) -> U,
          U: IntoFuture<Item=S::Item>,
{
    OrElse {
        stream: s,
        future: None,
        f: f,
    }
}

// Forwarding impl of Sink from the underlying stream
impl<S, F, U> ::sink::Sink for OrElse<S, F, U>
    where S: ::sink::Sink, U: IntoFuture
{
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;
}

impl<S, F, U, TaskT> ::sink::PollableSink<TaskT> for OrElse<S, F, U>
    where S: ::sink::PollableSink<TaskT>, U: IntoFuture
{
    fn start_send(&mut self, task: &mut TaskT, item: S::SinkItem) -> ::StartSend<S::SinkItem, S::SinkError> {
        self.stream.start_send(task, item)
    }

    fn poll_complete(&mut self, task: &mut TaskT) -> Poll<(), S::SinkError> {
        self.stream.poll_complete(task)
    }

    fn close(&mut self, task: &mut TaskT) -> Poll<(), S::SinkError> {
        self.stream.close(task)
    }
}

impl<S, F, U> Stream for OrElse<S, F, U>
    where S: Stream,
          F: FnMut(S::Error) -> U,
          U: IntoFuture<Item=S::Item>,
{
    type Item = S::Item;
    type Error = U::Error;
}

impl<S, F, U, TaskT> PollableStream<TaskT> for OrElse<S, F, U>
    where S: PollableStream<TaskT>,
          F: FnMut(S::Error) -> U,
          U: IntoFuture<Item=S::Item>,
          U::Future: Pollable<TaskT>
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<S::Item>, U::Error> {
        if self.future.is_none() {
            let item = match self.stream.poll(task) {
                Ok(Async::Ready(e)) => return Ok(Async::Ready(e)),
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(e) => e,
            };
            self.future = Some((self.f)(item).into_future());
        }
        assert!(self.future.is_some());
        match self.future.as_mut().unwrap().poll(task) {
            Ok(Async::Ready(e)) => {
                self.future = None;
                Ok(Async::Ready(Some(e)))
            }
            Err(e) => {
                self.future = None;
                Err(e)
            }
            Ok(Async::NotReady) => Ok(Async::NotReady)
        }
    }
}
