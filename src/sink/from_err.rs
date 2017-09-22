use core::marker::PhantomData;

use {Sink, Poll, StartSend, PollableSink};

/// A sink combinator to change the error type of a sink.
///
/// This is created by the `Sink::from_err` method.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct SinkFromErr<S, E> {
    sink: S,
    f: PhantomData<E>
}

pub fn new<S, E>(sink: S) -> SinkFromErr<S, E>
    where S: Sink
{
    SinkFromErr {
        sink: sink,
        f: PhantomData
    }
}

impl<S, E> SinkFromErr<S, E> {
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
}

impl<S, E> Sink for SinkFromErr<S, E>
    where S: Sink,
          E: From<S::SinkError>
{
    type SinkItem = S::SinkItem;
    type SinkError = E;
}

impl<S, E, TaskT> PollableSink<TaskT> for SinkFromErr<S, E>
    where S: PollableSink<TaskT>,
          E: From<S::SinkError>
{
    fn start_send(&mut self, task: &mut TaskT, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.sink.start_send(task, item).map_err(|e| e.into())
    }

    fn poll_complete(&mut self, task: &mut TaskT) -> Poll<(), Self::SinkError> {
        self.sink.poll_complete(task).map_err(|e| e.into())
    }

    fn close(&mut self, task: &mut TaskT) -> Poll<(), Self::SinkError> {
        self.sink.close(task).map_err(|e| e.into())
    }
}

impl<S: ::stream::Stream, E> ::stream::Stream for SinkFromErr<S, E> {
    type Item = S::Item;
    type Error = S::Error;
}

impl<S, E, TaskT> ::stream::PollableStream<TaskT> for SinkFromErr<S, E>
    where S: ::stream::PollableStream<TaskT>
{
    fn poll(&mut self, task: &mut TaskT) -> Poll<Option<S::Item>, S::Error> {
        self.sink.poll(task)
    }
}
