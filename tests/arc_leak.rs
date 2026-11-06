//! Real-time timer test. Kept in its own binary so it does not share the
//! paused clock.

use std::time::Duration;

use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

#[cfg(browser)]
wasm_bindgen_test_configure!(run_in_browser);

/// A long sleep stays pending while shorter sleeps are started.
///
/// That used to arm a new `setTimeout` for every sleep and never cancel the
/// previous one, so the timer `Arc` climbed past 20 and scheduling stopped.
#[cfg(feature = "tokio")]
#[wasm_bindgen_test]
async fn arc_leak_while_long_sleep_is_pending() {
    use wasmtimer::std::Instant;
    use wasmtimer::tokio::sleep;

    let short = Duration::from_millis(50);
    let long = Duration::from_secs(50);
    // Past the old `Arc::strong_count > 20` bailout. Started together so this
    // binary stays inside the browser harness timeout.
    const SHORT_SLEEPS: usize = 30;

    let long_sleep = sleep(long);
    let mut short_sleeps: Vec<_> = (0..SHORT_SLEEPS).map(|_| sleep(short)).collect();

    let started = Instant::now();
    short_sleeps.remove(0).await;
    let elapsed = Instant::now() - started;

    assert!(
        !long_sleep.is_elapsed(),
        "the long sleep fired while the short ones were still starting"
    );
    assert!(
        elapsed < Duration::from_secs(10),
        "the short sleep did not fire while the long one was pending: {elapsed:?}"
    );
}
