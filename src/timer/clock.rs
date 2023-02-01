use crate::std::Instant;
use std::sync::atomic::{AtomicPtr, Ordering::SeqCst};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static CLOCK: AtomicPtr<Mutex<Inner>> = AtomicPtr::new(EMPTY_CLOCK);
const EMPTY_CLOCK: *mut Mutex<Inner> = std::ptr::null_mut();

pub(crate) fn clock() -> Clock {
    let mut clock = CLOCK.load(std::sync::atomic::Ordering::SeqCst);

    if clock == EMPTY_CLOCK {
        let clock_new = Clock::new(false);
        CLOCK
            .compare_exchange(EMPTY_CLOCK, clock_new.into_raw(), SeqCst, SeqCst)
            .unwrap();
        clock = CLOCK.load(SeqCst);
    }

    assert!(clock != EMPTY_CLOCK);
    unsafe {
        let clock = Clock::from_raw(clock);
        let ret = clock.clone();
        drop(clock.into_raw());
        return ret;
    }
}

#[derive(Debug)]
struct Inner {
    base: Instant,
    unfrozen: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct Clock {
    inner: Arc<Mutex<Inner>>,
}

impl Clock {
    fn into_raw(self) -> *mut Mutex<Inner> {
        Arc::into_raw(self.inner.clone()) as *mut Mutex<Inner>
    }

    unsafe fn from_raw(raw: *mut Mutex<Inner>) -> Clock {
        let inner = Arc::from_raw(raw);
        Clock { inner }
    }
}

pub(crate) fn now() -> Instant {
    clock().now()
}

impl Clock {
    pub(crate) fn new(start_paused: bool) -> Clock {
        let now = Instant::now_js();

        let clock = Clock {
            inner: Arc::new(Mutex::new(Inner {
                base: now,
                unfrozen: Some(now)
            })),
        };

        if start_paused {
            clock.pause();
        }

        clock
    }

    pub(crate) fn resume(&self) {
        let mut inner = self.inner.lock().unwrap();

        if inner.unfrozen.is_some() {
            panic!("time is not frozen");
        }

        inner.unfrozen = Some(Instant::now());
    }

    pub(crate) fn paused(&self) -> bool {
        let inner = self.inner.lock().unwrap();

        inner.unfrozen.is_none()
    }

    #[track_caller]
    pub(crate) fn pause(&self) {
        let mut inner = self.inner.lock().unwrap();

        let elapsed = inner
            .unfrozen
            .as_ref()
            .expect("time is already frozen")
            .elapsed();
        inner.base += elapsed;
        inner.unfrozen = None;
    }

    #[track_caller]
    pub(crate) fn advance(&self, duration: Duration) {
        let mut inner = self.inner.lock().unwrap();

        if inner.unfrozen.is_some() {
            panic!("time is not frozen");
        }

        inner.base += duration;
    }

    pub(crate) fn now(&self) -> Instant {
        let inner = self.inner.lock().unwrap();

        let mut ret = inner.base;

        if let Some(unfrozen) = inner.unfrozen {
            ret += unfrozen.elapsed();
        }

        ret
    }
}
