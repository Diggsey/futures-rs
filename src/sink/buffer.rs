use std::collections::VecDeque;

use {Poll, Async, PollableStream, PollableSink};
use {StartSend, AsyncSink};
use sink::Sink;
use stream::Stream;

/// Sink for the `Sink::buffer` combinator, which buffers up to some fixed
/// number of values when the underlying sink is unable to accept them.
#[derive(Debug)]
#[must_use = "sinks do nothing unless polled"]
pub struct Buffer<S: Sink> {
    sink: S,
    buf: VecDeque<S::SinkItem>,

    // Track capacity separately from the `VecDeque`, which may be rounded up
    cap: usize,
}

pub fn new<S: Sink>(sink: S, amt: usize) -> Buffer<S> {
    Buffer {
        sink: sink,
        buf: VecDeque::with_capacity(amt),
        cap: amt,
    }
}

impl<S: Sink> Buffer<S> {
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

    fn try_empty_buffer<TaskT>(&mut self, task: &mut TaskT) -> Poll<(), S::SinkError> where S: PollableSink<TaskT> {
        while let Some(item) = self.buf.pop_front() {
            if let AsyncSink::NotReady(item) = self.sink.start_send(task, item)? {
                self.buf.push_front(item);

                // ensure that we attempt to complete any pushes we've started
                self.sink.poll_complete(task)?;

                return Ok(Async::NotReady);
            }
        }

        Ok(Async::Ready(()))
    }
}

// Forwarding impl of Stream from the underlying sink
impl<S> Stream for Buffer<S> where S: Sink + Stream {
    type Item = S::Item;
    type Error = S::Error;
}

// Forwarding impl of Stream from the underlying sink
impl<S, TaskT> PollableStream<TaskT> for Buffer<S> where S: PollableStream<TaskT> + Sink {
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<S::Item>, S::Error> {
        self.sink.poll(task)
    }
}

impl<S: Sink> Sink for Buffer<S> {
    type SinkItem = S::SinkItem;
    type SinkError = S::SinkError;
}

impl<S, TaskT> PollableSink<TaskT> for Buffer<S> where S: PollableSink<TaskT> {
    fn start_send(&mut self, task: &mut TaskT, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.try_empty_buffer(task)?;
        if self.buf.len() > self.cap {
            return Ok(AsyncSink::NotReady(item));
        }
        self.buf.push_back(item);
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self, task: &mut TaskT) -> Poll<(), Self::SinkError> {
        try_ready!(self.try_empty_buffer(task));
        debug_assert!(self.buf.is_empty());
        self.sink.poll_complete(task)
    }

    fn close(&mut self, task: &mut TaskT) -> Poll<(), Self::SinkError> {
        if self.buf.len() > 0 {
            try_ready!(self.try_empty_buffer(task));
        }
        assert_eq!(self.buf.len(), 0);
        self.sink.close(task)
    }
}
