use std::{task::Poll, time::Duration};

use futures::Future;
use pin_project_lite::pin_project;

use crate::std::Instant;

use super::{error::Elapsed, Sleep};

pin_project! {
    pub struct Timeout<T> {
        #[pin]
        delay: Sleep,
        #[pin]
        future: T,
    }
}

impl<F> Timeout<F>
where
    F: Future,
{
    pub(crate) fn new(dur: Duration, fut: F) -> Timeout<F> {
        Timeout {
            delay: Sleep::new(dur),
            future: fut,
        }
    }

    pub(crate) fn new_at(at: Instant, fut: F) -> Timeout<F> {
        Timeout {
            delay: Sleep::new_at(at),
            future: fut,
        }
    }

    pub fn get_ref(&self) -> &F {
        &self.future
    }

    pub fn get_mut(&mut self) -> &mut F {
        &mut self.future
    }

    pub fn into_inner(self) -> F {
        self.future
    }
}

impl<T> Future for Timeout<T>
where
    T: Future,
{
    type Output = Result<T::Output, Elapsed>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.future.poll(cx) {
            Poll::Pending => {}
            Poll::Ready(other) => return Poll::Ready(Ok(other)),
        }

        if this.delay.poll(cx).is_ready() {
            Poll::Ready(Err(Elapsed::new()))
        } else {
            Poll::Pending
        }
    }
}

pub fn timeout<F>(duration: Duration, future: F) -> Timeout<F>
where
    F: Future,
{
    Timeout::new(duration, future)
}

pub fn timeout_at<F>(deadline: Instant, future: F) -> Timeout<F>
where
    F: Future,
{
    Timeout::new_at(deadline, future)
}
