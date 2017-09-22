use {Async, Poll, PollableStream};
use stream::{Stream, Fuse};

/// A `Stream` that implements a `peek` method.
///
/// The `peek` method can be used to retrieve a reference
/// to the next `Stream::Item` if available. A subsequent
/// call to `poll` will return the owned item.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Peekable<S: Stream> {
    stream: Fuse<S>,
    peeked: Option<S::Item>,
}


pub fn new<S: Stream>(stream: S) -> Peekable<S> {
    Peekable {
        stream: stream.fuse(),
        peeked: None
    }
}

// Forwarding impl of Sink from the underlying stream
impl<S> ::sink::Sink for Peekable<S>
    where S: ::sink::Sink + Stream
{
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;
}

impl<S, TaskT> ::sink::PollableSink<TaskT> for Peekable<S>
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

impl<S: Stream> Stream for Peekable<S> {
    type Item = S::Item;
    type Error = S::Error;
}

impl<S, TaskT> PollableStream<TaskT> for Peekable<S> where S: PollableStream<TaskT> {
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(item) = self.peeked.take() {
            return Ok(Async::Ready(Some(item)))
        }
        self.stream.poll(task)
    }
}


impl<S: Stream> Peekable<S> {
    /// Peek retrieves a reference to the next item in the stream.
    ///
    /// This method polls the underlying stream and return either a reference
    /// to the next item if the stream is ready or passes through any errors.
    pub fn peek<TaskT>(&mut self, task: &mut TaskT) -> Poll<Option<&S::Item>, S::Error> where S: PollableStream<TaskT> {
        if self.peeked.is_some() {
            return Ok(Async::Ready(self.peeked.as_ref()))
        }
        match try_ready!(self.poll(task)) {
            None => Ok(Async::Ready(None)),
            Some(item) => {
                self.peeked = Some(item);
                Ok(Async::Ready(self.peeked.as_ref()))
            }
        }
    }
}
