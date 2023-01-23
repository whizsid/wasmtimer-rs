use pin_utils::unsafe_pinned;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::prelude::*;

use crate::std::Instant;
use crate::timer::TimerHandle;
use crate::tokio::Sleep;

/// A stream representing notifications at fixed interval
///
/// Intervals are created through the `Interval::new` or
/// `Interval::new_at` methods indicating when a first notification
/// should be triggered and when it will be repeated.
///
/// Note that intervals are not intended for high resolution timers, but rather
/// they will likely fire some granularity after the exact instant that they're
/// otherwise indicated to fire at.
#[derive(Debug)]
pub struct Interval {
    sleep: Sleep,
    interval: Duration,
}

impl Interval {
    unsafe_pinned!(sleep: Sleep);

    /// Creates a new interval which will fire at `dur` time into the future,
    /// and will repeat every `dur` interval after
    ///
    /// The returned object will be bound to the default timer for this thread.
    /// The default timer will be spun up in a helper thread on first use.
    pub fn new(dur: Duration) -> Interval {
        Interval::new_at(Instant::now() + dur, dur)
    }

    /// Creates a new interval which will fire at the time specified by `at`,
    /// and then will repeat every `dur` interval after
    ///
    /// The returned object will be bound to the default timer for this thread.
    /// The default timer will be spun up in a helper thread on first use.
    pub fn new_at(at: Instant, dur: Duration) -> Interval {
        Interval {
            sleep: Sleep::new_at(at),
            interval: dur,
        }
    }

    /// Creates a new interval which will fire at the time specified by `at`,
    /// and then will repeat every `dur` interval after
    ///
    /// The returned object will be bound to the timer specified by `handle`.
    pub fn new_handle(at: Instant, dur: Duration, handle: TimerHandle) -> Interval {
        Interval {
            sleep: Sleep::new_handle(at, handle),
            interval: dur,
        }
    }
}

impl Stream for Interval {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let test = Pin::new(&mut *self);
        if Pin::new(&mut *self).sleep().poll(cx).is_pending() {
            return Poll::Pending;
        }
        let next = next_interval(self.sleep.deadline(), Instant::now(), self.interval);
        self.sleep().reset(next);
        Poll::Ready(Some(()))
    }
}

/// Converts Duration object to raw nanoseconds if possible
///
/// This is useful to divide intervals.
///
/// While technically for large duration it's impossible to represent any
/// duration as nanoseconds, the largest duration we can represent is about
/// 427_000 years. Large enough for any interval we would use or calculate in
/// tokio.
fn duration_to_nanos(dur: Duration) -> Option<u64> {
    dur.as_secs()
        .checked_mul(1_000_000_000)
        .and_then(|v| v.checked_add(dur.subsec_nanos() as u64))
}

fn next_interval(prev: Instant, now: Instant, interval: Duration) -> Instant {
    let new = prev + interval;
    if new > now {
        return new;
    } else {
        let spent_ns =
            duration_to_nanos(now.duration_since(prev)).expect("interval should be expired");
        let interval_ns =
            duration_to_nanos(interval).expect("interval is less that 427 thousand years");
        let mult = spent_ns / interval_ns + 1;
        assert!(
            mult < (1 << 32),
            "can't skip more than 4 billion intervals of {:?} \
             (trying to skip {})",
            interval,
            mult
        );
        return prev + interval * (mult as u32);
    }
}
