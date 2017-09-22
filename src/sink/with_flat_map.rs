use core::marker::PhantomData;

use {Poll, Async, StartSend, AsyncSink, PollableSink, PollableStream};
use sink::Sink;
use stream::Stream;

/// Sink for the `Sink::with_flat_map` combinator, chaining a computation that returns an iterator
/// to run prior to pushing a value into the underlying sink
#[derive(Debug)]
#[must_use = "sinks do nothing unless polled"]
pub struct WithFlatMap<S, U, F, St>
where
    S: Sink,
    F: FnMut(U) -> St,
    St: Stream<Item = S::SinkItem, Error=S::SinkError>,
{
    sink: S,
    f: F,
    stream: Option<St>,
    buffer: Option<S::SinkItem>,
    _phantom: PhantomData<fn(U)>,
}

pub fn new<S, U, F, St>(sink: S, f: F) -> WithFlatMap<S, U, F, St>
where
    S: Sink,
    F: FnMut(U) -> St,
    St: Stream<Item = S::SinkItem, Error=S::SinkError>,
{
    WithFlatMap {
        sink: sink,
        f: f,
        stream: None,
        buffer: None,
        _phantom: PhantomData,
    }
}

impl<S, U, F, St> WithFlatMap<S, U, F, St>
where
    S: Sink,
    F: FnMut(U) -> St,
    St: Stream<Item = S::SinkItem, Error=S::SinkError>,
{
    /// Get a shared reference to the inner sink.
    pub fn get_ref(&self) -> &S {
        &self.sink
    }

    /// Get a mutable reference to the inner sink.
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.sink
    }

    /// Consumes this combinator, returning the underlying sink.
    ///
    /// Note that this may discard intermediate state of this combinator, so
    /// care should be taken to avoid losing resources when this is called.
    pub fn into_inner(self) -> S {
        self.sink
    }

    fn try_empty_stream<TaskT>(&mut self, task: &mut TaskT) -> Poll<(), S::SinkError> where S: PollableSink<TaskT>, St: PollableStream<TaskT> {
        if let Some(x) = self.buffer.take() {
            if let AsyncSink::NotReady(x) = self.sink.start_send(task, x)? {
                self.buffer = Some(x);
                return Ok(Async::NotReady);
            }
        }
        if let Some(mut stream) = self.stream.take() {
            while let Some(x) = try_ready!(stream.poll(task)) {
                if let AsyncSink::NotReady(x) = self.sink.start_send(task, x)? {
                    self.stream = Some(stream);
                    self.buffer = Some(x);
                    return Ok(Async::NotReady);
                }
            }
        }
        Ok(Async::Ready(()))
    }
}

impl<S, U, F, St> Stream for WithFlatMap<S, U, F, St>
where
    S: Stream + Sink,
    F: FnMut(U) -> St,
    St: Stream<Item = S::SinkItem, Error=S::SinkError>,
{
    type Item = S::Item;
    type Error = S::Error;
}

impl<S, U, F, St, TaskT> PollableStream<TaskT> for WithFlatMap<S, U, F, St>
where
    S: PollableStream<TaskT> + Sink,
    F: FnMut(U) -> St,
    St: Stream<Item = S::SinkItem, Error=S::SinkError>,
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<S::Item>, S::Error> {
        self.sink.poll(task)
    }
}

impl<S, U, F, St> Sink for WithFlatMap<S, U, F, St>
where
    S: Sink,
    F: FnMut(U) -> St,
    St: Stream<Item = S::SinkItem, Error=S::SinkError>,
{
    type SinkItem = U;
    type SinkError = S::SinkError;
}

impl<S, U, F, St, TaskT> PollableSink<TaskT> for WithFlatMap<S, U, F, St>
where
    S: PollableSink<TaskT>,
    F: FnMut(U) -> St,
    St: PollableStream<TaskT, Item = S::SinkItem, Error=S::SinkError>,
{
    fn start_send(&mut self, task: &mut TaskT, i: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        if self.try_empty_stream(task)?.is_not_ready() {
            return Ok(AsyncSink::NotReady(i));
        }
        assert!(self.stream.is_none());
        self.stream = Some((self.f)(i));
        self.try_empty_stream(task)?;
        Ok(AsyncSink::Ready)
    }
    fn poll_complete(&mut self, task: &mut TaskT) -> Poll<(), Self::SinkError> {
        if self.try_empty_stream(task)?.is_not_ready() {
            return Ok(Async::NotReady);
        }
        self.sink.poll_complete(task)
    }
    fn close(&mut self, task: &mut TaskT) -> Poll<(), Self::SinkError> {
        if self.try_empty_stream(task)?.is_not_ready() {
            return Ok(Async::NotReady);
        }
        assert!(self.stream.is_none());
        self.sink.close(task)
    }
}
