use {Async, Poll, PollableStream};
use stream::Stream;

/// A stream combinator which returns a maximum number of elements.
///
/// This structure is produced by the `Stream::take` method.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Take<S> {
    stream: S,
    remaining: u64,
}

pub fn new<S>(s: S, amt: u64) -> Take<S>
    where S: Stream,
{
    Take {
        stream: s,
        remaining: amt,
    }
}

impl<S> Take<S> {
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
impl<S> ::sink::Sink for Take<S>
    where S: ::sink::Sink + Stream
{
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;
}

impl<S, TaskT> ::sink::PollableSink<TaskT> for Take<S>
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

impl<S> Stream for Take<S>
    where S: Stream,
{
    type Item = S::Item;
    type Error = S::Error;
}

impl<S, TaskT> PollableStream<TaskT> for Take<S>
    where S: PollableStream<TaskT>,
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<S::Item>, S::Error> {
        if self.remaining == 0 {
            Ok(Async::Ready(None))
        } else {
            let next = try_ready!(self.stream.poll(task));
            match next {
                Some(_) => self.remaining -= 1,
                None => self.remaining = 0,
            }
            Ok(Async::Ready(next))
        }
    }
}
