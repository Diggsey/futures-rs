use {Poll, Async, PollableStream};
use stream::Stream;

/// A stream which "fuse"s a stream once it's terminated.
///
/// Normally streams can behave unpredictably when used after they have already
/// finished, but `Fuse` continues to return `None` from `poll` forever when
/// finished.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Fuse<S> {
    stream: S,
    done: bool,
}

// Forwarding impl of Sink from the underlying stream
impl<S> ::sink::Sink for Fuse<S>
    where S: ::sink::Sink
{
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;
}

// Forwarding impl of Sink from the underlying stream
impl<S, TaskT> ::sink::PollableSink<TaskT> for Fuse<S>
    where S: ::sink::PollableSink<TaskT>
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

pub fn new<S: Stream>(s: S) -> Fuse<S> {
    Fuse { stream: s, done: false }
}

impl<S: Stream> Stream for Fuse<S> {
    type Item = S::Item;
    type Error = S::Error;
}

impl<S, TaskT> PollableStream<TaskT> for Fuse<S> where S: PollableStream<TaskT> {
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<S::Item>, S::Error> {
        if self.done {
            Ok(Async::Ready(None))
        } else {
            let r = self.stream.poll(task);
            if let Ok(Async::Ready(None)) = r {
                self.done = true;
            }
            r
        }
    }
}

impl<S> Fuse<S> {
    /// Returns whether the underlying stream has finished or not.
    ///
    /// If this method returns `true`, then all future calls to poll are
    /// guaranteed to return `None`. If this returns `false`, then the
    /// underlying stream is still in use.
    pub fn is_done(&self) -> bool {
        self.done
    }

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
