
use std::time::Duration;

use wasm_bindgen_test::{wasm_bindgen_test_configure, wasm_bindgen_test, console_log};
use wasmtimer::tokio::{sleep, pause, advance};

wasm_bindgen_test_configure!(run_in_browser);


#[wasm_bindgen_test]
async fn sleep_test() {
    console_log!("Pausing timer");
    pause();
    console_log!("Paused timer");
    console_log!("Sleeping");
    let sleeped = sleep(Duration::from_millis(1000));
    console_log!("Sleeped");
    assert!(!sleeped.is_elapsed());
    console_log!("Advancing");
    advance(Duration::from_millis(1005)).await;
    console_log!("Advanced");
    assert!(sleeped.is_elapsed());
}
