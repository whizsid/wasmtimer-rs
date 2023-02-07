use std::{sync::Once, time::Duration};

use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
use wasmtimer::tokio::{advance, pause};

wasm_bindgen_test_configure!(run_in_browser);

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        pause();
    });
}

pub mod sleep_tests {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    use super::*;
    use futures::{task::noop_waker_ref, Future};
    use wasmtimer::{tokio::sleep, std::Instant};

    #[wasm_bindgen_test]
    async fn is_elapsed_test() {
        initialize();
        let slept = sleep(Duration::from_millis(1000));
        assert!(!slept.is_elapsed());
        advance(Duration::from_millis(1005)).await;
        assert!(slept.is_elapsed());
    }

    #[wasm_bindgen_test]
    async fn poll_test() {
        initialize();
        let waker = noop_waker_ref();
        let mut cx = Context::from_waker(waker);

        let mut slept = sleep(Duration::from_millis(1000));
        assert_eq!(Pin::new(&mut slept).poll(&mut cx), Poll::Pending);
        advance(Duration::from_millis(1005)).await;
        assert_eq!(Pin::new(&mut slept).poll(&mut cx), Poll::Ready(()));
    }

    #[wasm_bindgen_test]
    async fn reset_before_exec_test() {
        initialize();
        let waker = noop_waker_ref();
        let mut cx = Context::from_waker(waker);

        let mut slept = sleep(Duration::from_millis(1000));
        Pin::new(&mut slept).reset(Instant::now() + Duration::from_millis(2000));
        assert_eq!(Pin::new(&mut slept).poll(&mut cx), Poll::Pending);
        advance(Duration::from_millis(1005)).await;
        assert_eq!(Pin::new(&mut slept).poll(&mut cx), Poll::Pending);
        advance(Duration::from_millis(1005)).await;
        assert_eq!(Pin::new(&mut slept).poll(&mut cx), Poll::Ready(()));
    }

    #[wasm_bindgen_test]
    async fn reset_after_exec_test() {
        initialize();
        let waker = noop_waker_ref();
        let mut cx = Context::from_waker(waker);

        let mut slept = sleep(Duration::from_millis(1000));
        assert_eq!(Pin::new(&mut slept).poll(&mut cx), Poll::Pending);
        advance(Duration::from_millis(1005)).await;
        assert_eq!(Pin::new(&mut slept).poll(&mut cx), Poll::Ready(()));
        Pin::new(&mut slept).reset(Instant::now() + Duration::from_millis(1500));
        assert_eq!(Pin::new(&mut slept).poll(&mut cx), Poll::Pending);
        advance(Duration::from_millis(1505)).await;
        assert_eq!(Pin::new(&mut slept).poll(&mut cx), Poll::Ready(()));
    }
}
