mod utils;

use std::time::Duration;

use wasm_bindgen::prelude::*;
use wasmtimer::std::Instant;
use wasmtimer::tokio::{interval, sleep, sleep_until, timeout};
use web_sys::console::log_1;
use web_sys::window;

#[wasm_bindgen]
pub async fn sleep_test() {
    log_1(&JsValue::from_str("Sleeping Rust"));
    sleep(Duration::from_secs(3)).await;
    log_1(&JsValue::from_str("Slept Rust"));
}

#[wasm_bindgen]
pub async fn interval_test() {
    log_1(&JsValue::from_str("Interval Rust 1"));
    let mut interval = interval(Duration::from_secs(3));
    interval.tick().await;
    log_1(&JsValue::from_str("Interval Rust 2"));
    interval.tick().await;
    log_1(&JsValue::from_str("Interval Rust 3"));
    interval.tick().await;
    log_1(&JsValue::from_str("Interval Rust 4"));
    interval.tick().await;
    log_1(&JsValue::from_str("Interval Rust 5"));
    drop(interval);
}

#[wasm_bindgen]
pub async fn timeout_test() {
    log_1(&JsValue::from_str("Timeout Start Rust"));
    let result = timeout(Duration::from_secs(1), async {
        log_1(&JsValue::from_str("Timeout Callback Rust"));
        sleep(Duration::from_secs(3)).await;
        log_1(&JsValue::from_str("Timeout Failed 1 Rust"));
    })
    .await;
    match result {
        Ok(_) => {
            log_1(&JsValue::from_str("Timeout Failed 2 Rust"));
        }
        Err(e) => {
            log_1(&JsValue::from_str(&format!(
                "Timeout Success. Error:- {:?}",
                e
            )));
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["globalThis", "performance"])]
    fn now() -> f64;
}

#[wasm_bindgen]
pub async fn arc_leak_test() {
    let duration = Duration::from_millis(50);
    for _ in 0..500 {
        let deadline = Instant::now() + duration;
        sleep_until(deadline).await;
        let ms = now();
        log_1(&JsValue::from_f64(ms));
    }
}
