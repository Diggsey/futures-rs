use {Async, Poll, PollableStream};
use stream::Stream;

/// A combinator used to filter the results of a stream and simultaneously map
/// them to a different type.
///
/// This structure is returned by the `Stream::filter_map` method.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct FilterMap<S, F> {
    stream: S,
    f: F,
}

pub fn new<S, F, B>(s: S, f: F) -> FilterMap<S, F>
    where S: Stream,
          F: FnMut(S::Item) -> Option<B>,
{
    FilterMap {
        stream: s,
        f: f,
    }
}

impl<S, F> FilterMap<S, F> {
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
impl<S, F> ::sink::Sink for FilterMap<S, F>
    where S: ::sink::Sink
{
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;
}

impl<S, F, TaskT> ::sink::PollableSink<TaskT> for FilterMap<S, F>
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

impl<S, F, B> Stream for FilterMap<S, F>
    where S: Stream,
          F: FnMut(S::Item) -> Option<B>,
{
    type Item = B;
    type Error = S::Error;
}

impl<S, F, B, TaskT> PollableStream<TaskT> for FilterMap<S, F>
    where S: PollableStream<TaskT>,
          F: FnMut(S::Item) -> Option<B>,
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<B>, S::Error> {
        loop {
            match try_ready!(self.stream.poll(task)) {
                Some(e) => {
                    if let Some(e) = (self.f)(e) {
                        return Ok(Async::Ready(Some(e)))
                    }
                }
                None => return Ok(Async::Ready(None)),
            }
        }
    }
}
