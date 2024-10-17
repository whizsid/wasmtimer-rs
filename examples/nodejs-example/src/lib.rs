mod utils;

use std::time::Duration;

use tokio;
use wasm_bindgen::prelude::*;
use wasmtimer::tokio::{interval, sleep, timeout};
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
pub async fn tokio_macros_test() {
    log_1(&JsValue::from_str("Tokio macros start Rust"));
    async fn some_async_work_1() {
        sleep(Duration::from_secs(1)).await;
    }
    async fn some_async_work_2() {
        sleep(Duration::from_secs(2)).await;
    }
    tokio::select! {
        _ = some_async_work_1() => {
            log_1(&JsValue::from_str("Tokio macros 1 second Rust"));
        },
        _ = some_async_work_2() => {
            log_1(&JsValue::from_str("Tokio macros 2 second Rust"));
        }
    }
    log_1(&JsValue::from_str("Tokio macros end Rust"));
}
