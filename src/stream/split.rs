use std::any::Any;
use std::error::Error;
use std::fmt;

use {StartSend, Sink, Stream, Poll, Async, AsyncSink, PollableStream, PollableSink};
use sync::BiLock;

/// A `Stream` part of the split pair
#[derive(Debug)]
pub struct SplitStream<S>(BiLock<S>);

impl<S> SplitStream<S> {
    /// Attempts to put the two "halves" of a split `Stream + Sink` back
    /// together. Succeeds only if the `SplitStream<S>` and `SplitSink<S>` are
    /// a matching pair originating from the same call to `Stream::split`.
    pub fn reunite(self, other: SplitSink<S>) -> Result<S, ReuniteError<S>> {
        other.reunite(self)
    }
}

impl<S: Stream> Stream for SplitStream<S> {
    type Item = S::Item;
    type Error = S::Error;
}

impl<S, TaskT> PollableStream<TaskT> for SplitStream<S> where S: PollableStream<TaskT> {
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<S::Item>, S::Error> {
        match self.0.poll_lock() {
            Async::Ready(mut inner) => inner.poll(task),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

/// A `Sink` part of the split pair
#[derive(Debug)]
pub struct SplitSink<S>(BiLock<S>);

impl<S> SplitSink<S> {
    /// Attempts to put the two "halves" of a split `Stream + Sink` back
    /// together. Succeeds only if the `SplitStream<S>` and `SplitSink<S>` are
    /// a matching pair originating from the same call to `Stream::split`.
    pub fn reunite(self, other: SplitStream<S>) -> Result<S, ReuniteError<S>> {
        self.0.reunite(other.0).map_err(|err| {
            ReuniteError(SplitSink(err.0), SplitStream(err.1))
        })
    }
}

impl<S: Sink> Sink for SplitSink<S> {
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;
}

impl<S, TaskT> PollableSink<TaskT> for SplitSink<S> where S: PollableSink<TaskT> {
    fn start_send(&mut self, task: &mut TaskT, item: S::SinkItem)
        -> StartSend<S::SinkItem, S::SinkError>
    {
        match self.0.poll_lock() {
            Async::Ready(mut inner) => inner.start_send(task, item),
            Async::NotReady => Ok(AsyncSink::NotReady(item)),
        }
    }

    fn poll_complete(&mut self, task: &mut TaskT) -> Poll<(), S::SinkError> {
        match self.0.poll_lock() {
            Async::Ready(mut inner) => inner.poll_complete(task),
            Async::NotReady => Ok(Async::NotReady),
        }
    }

    fn close(&mut self, task: &mut TaskT) -> Poll<(), S::SinkError> {
        match self.0.poll_lock() {
            Async::Ready(mut inner) => inner.close(task),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

pub fn split<S: Stream + Sink>(s: S) -> (SplitSink<S>, SplitStream<S>) {
    let (a, b) = BiLock::new(s);
    let read = SplitStream(a);
    let write = SplitSink(b);
    (write, read)
}

/// Error indicating a `SplitSink<S>` and `SplitStream<S>` were not two halves
/// of a `Stream + Split`, and thus could not be `reunite`d.
pub struct ReuniteError<T>(pub SplitSink<T>, pub SplitStream<T>);

impl<T> fmt::Debug for ReuniteError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("ReuniteError")
            .field(&"...")
            .finish()
    }
}

impl<T> fmt::Display for ReuniteError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "tried to reunite a SplitStream and SplitSink that don't form a pair")
    }
}

impl<T: Any> Error for ReuniteError<T> {
    fn description(&self) -> &str {
        "tried to reunite a SplitStream and SplitSink that don't form a pair"
    }
}
