use {Async, Poll, PollableStream};
use stream::Stream;

/// A stream combinator used to filter the results of a stream and only yield
/// some values.
///
/// This structure is produced by the `Stream::filter` method.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Filter<S, F> {
    stream: S,
    f: F,
}

pub fn new<S, F>(s: S, f: F) -> Filter<S, F>
    where S: Stream,
          F: FnMut(&S::Item) -> bool,
{
    Filter {
        stream: s,
        f: f,
    }
}

impl<S, F> Filter<S, F> {
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
impl<S, F> ::sink::Sink for Filter<S, F>
    where S: ::sink::Sink
{
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;
}

impl<S, F, TaskT> ::sink::PollableSink<TaskT> for Filter<S, F>
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

impl<S, F> Stream for Filter<S, F>
    where S: Stream,
          F: FnMut(&S::Item) -> bool,
{
    type Item = S::Item;
    type Error = S::Error;
}

impl<S, F, TaskT> PollableStream<TaskT> for Filter<S, F>
    where S: PollableStream<TaskT>,
          F: FnMut(&S::Item) -> bool,
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<S::Item>, S::Error> {
        loop {
            match try_ready!(self.stream.poll(task)) {
                Some(e) => {
                    if (self.f)(&e) {
                        return Ok(Async::Ready(Some(e)))
                    }
                }
                None => return Ok(Async::Ready(None)),
            }
        }
    }
}
