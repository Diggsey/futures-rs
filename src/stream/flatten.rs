use {Poll, Async, PollableStream};
use stream::Stream;

/// A combinator used to flatten a stream-of-streams into one long stream of
/// elements.
///
/// This combinator is created by the `Stream::flatten` method.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Flatten<S>
    where S: Stream,
{
    stream: S,
    next: Option<S::Item>,
}

pub fn new<S>(s: S) -> Flatten<S>
    where S: Stream,
          S::Item: Stream,
          <S::Item as Stream>::Error: From<S::Error>,
{
    Flatten {
        stream: s,
        next: None,
    }
}

impl<S: Stream> Flatten<S> {
    /// Acquires a reference to the underlying stream that this combinator is
    /// pulling from.
    pub fn get_ref(&self) -> &S {
        &self.stream
    }

    /// Acquires a mutable reference to the underlying stream that this
    /// combinator is pulling from.
    ///
    /// Note that care must be taken to avoid tampering with the state of the
    /// stream which may otherwise confuse this combinator.
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    /// Consumes this combinator, returning the underlying stream.
    ///
    /// Note that this may discard intermediate state of this combinator, so
    /// care should be taken to avoid losing resources when this is called.
    pub fn into_inner(self) -> S {
        self.stream
    }
}

// Forwarding impl of Sink from the underlying stream
impl<S> ::sink::Sink for Flatten<S>
    where S: ::sink::Sink + Stream
{
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;
}

impl<S, TaskT> ::sink::PollableSink<TaskT> for Flatten<S>
    where S: ::sink::PollableSink<TaskT> + Stream
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

impl<S> Stream for Flatten<S>
    where S: Stream,
          S::Item: Stream,
          <S::Item as Stream>::Error: From<S::Error>,
{
    type Item = <S::Item as Stream>::Item;
    type Error = <S::Item as Stream>::Error;
}

impl<S, TaskT> PollableStream<TaskT> for Flatten<S>
    where S: PollableStream<TaskT>,
          S::Item: PollableStream<TaskT>,
          <S::Item as Stream>::Error: From<S::Error>,
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if self.next.is_none() {
                match try_ready!(self.stream.poll(task)) {
                    Some(e) => self.next = Some(e),
                    None => return Ok(Async::Ready(None)),
                }
            }
            assert!(self.next.is_some());
            match self.next.as_mut().unwrap().poll(task) {
                Ok(Async::Ready(None)) => self.next = None,
                other => return other,
            }
        }
    }
}
