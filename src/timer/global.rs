use futures::task::{self, ArcWake};
use parking_lot::Mutex;
use std::cell::{Cell, RefCell};
use std::convert::TryFrom;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::time::Duration;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

use crate::js::{clear_timeout, set_timeout};
use crate::std::Instant;
use crate::timer::{Timer, TimerHandle};

// Handle returned by the outstanding `setTimeout`, if there is one.
// `JsValue` is not `Send`, so it cannot live next to the timer.
thread_local! {
    static ARMED_TIMEOUT: RefCell<Option<JsValue>> = const { RefCell::new(None) };
    // The callback below is on the stack. A wake from there only sets `WOKEN`;
    // arming another timeout here would be cleared by the callback's own reschedule.
    static IN_CALLBACK: Cell<bool> = const { Cell::new(false) };
    static WOKEN: Cell<bool> = const { Cell::new(false) };
}

struct InCallback;

impl InCallback {
    fn enter() -> Self {
        IN_CALLBACK.with(|flag| flag.set(true));
        Self
    }
}

impl Drop for InCallback {
    fn drop(&mut self) {
        IN_CALLBACK.with(|flag| flag.set(false));
    }
}

/// Starts a background task, creates a `Timer`, and returns a handle to it.
///
/// > **Note**: Contrary to the original `futures-timer` crate, we don't have
/// >           any `forget()` method, as the task is automatically considered
/// >           as "forgotten".
pub(crate) fn run() -> TimerHandle {
    let timer = Timer::new();
    let handle = timer.handle();
    schedule_callback(Arc::new(Mutex::new(timer)), Duration::new(0, 0));
    handle
}

fn disarm() {
    if let Some(handle) = ARMED_TIMEOUT.with(|slot| slot.borrow_mut().take()) {
        clear_timeout(&handle);
    }
}

/// Calls `Window::setTimeout` with the given `Duration`. The callback wakes up the timer and
/// processes everything.
fn schedule_callback(timer: Arc<Mutex<Timer>>, when: Duration) {
    if IN_CALLBACK.with(|flag| flag.get()) {
        WOKEN.with(|flag| flag.set(true));
        return;
    }

    // Cancel the previous callback before scheduling a new one.
    disarm();

    let cb = move || {
        // This callback is already running, so just forget the handle.
        ARMED_TIMEOUT.with(|slot| drop(slot.borrow_mut().take()));

        let sleep_dur = {
            let _in_callback = InCallback::enter();
            let mut timer_lock = timer.lock();

            // We start by polling the timer. If any new `Delay` is created, the waker will be used
            // to wake up this task pre-emptively. As such, we pass a `Waker` that calls
            // `schedule_callback` with a delay of `0`.
            let waker = task::waker(Arc::new(Waker {
                timer: timer.clone(),
            }));

            loop {
                WOKEN.with(|flag| flag.set(false));
                let _ = Future::poll(Pin::new(&mut *timer_lock), &mut Context::from_waker(&waker));
                let now = Instant::now();
                timer_lock.advance_to(now);
                if WOKEN.with(|flag| flag.get()) {
                    continue;
                }

                break timer_lock.next_event().map(|next_event| {
                    if next_event > now {
                        next_event - now
                    } else {
                        Duration::new(0, 0)
                    }
                });
            }
        };

        // A previous version of this function recursed in such a way as the strong count increased
        // From testing, this one doesn't (it remains at a constant 2)
        if Arc::strong_count(&timer) > 20 {
            return;
        }

        // A sleep was queued by a task we just woke. Poll it before arming the
        // next deadline, or a still-pending longer sleep would hide it.
        if WOKEN.with(|flag| flag.replace(false)) {
            schedule_callback(timer, Duration::new(0, 0));
            return;
        }

        if let Some(sleep) = sleep_dur {
            schedule_callback(timer, sleep);
        }
    };

    #[cfg(feature = "tokio-test-util")]
    if super::clock::clock().paused() {
        // Due work runs inline. A future deadline waits for `advance`, which
        // wakes us again. Calling `cb` here too used to recurse until the
        // strong-count guard above stopped scheduling.
        if when != Duration::new(0, 0) {
            return;
        }
        cb();
        return;
    }

    let handle = set_timeout(
        Closure::once_into_js(cb).unchecked_ref(),
        i32::try_from(when.as_millis()).unwrap_or(0),
    )
    .unwrap();
    ARMED_TIMEOUT.with(|slot| *slot.borrow_mut() = Some(handle));
}

struct Waker {
    timer: Arc<Mutex<Timer>>,
}

impl ArcWake for Waker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        schedule_callback(arc_self.timer.clone(), Duration::new(0, 0));
    }
}
