mod utils;

use std::time::Duration;

use wasm_bindgen::prelude::*;
use wasmtimer::tokio::sleep;
use wasmtimer::tokio::interval;
use web_sys::console::log_1;

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
