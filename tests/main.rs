use futures::StreamExt;
use spiderweb::{
    time::{Interval, Timeout, Instant, SystemTime}, task::sleep,
};
use std::time::Duration;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn interval() {
    let mut secs = 0;
    let int = Interval::new(
        || {
            secs += 1;
            secs
        },
        Duration::from_millis(250),
    );

    let mut int = int.take(5).enumerate();
    while let Some((i, x)) = int.next().await {
        assert_eq!(i + 1, x);
    }
}

#[wasm_bindgen_test]
async fn timeout() {
    let int = Timeout::new_async(
        async move { Timeout::new(|| 21, Duration::from_millis(500)).await },
        Duration::from_secs(1),
    );

    assert_eq!(int.await, 21)
}

#[wasm_bindgen_test]
async fn instant () {
    let time = Instant::now();
    sleep(Duration::from_secs(2)).await;
    assert_eq!(time.elapsed().as_secs(), 2);
}

#[wasm_bindgen_test]
async fn system_time () {
    let time = SystemTime::now();
    sleep(Duration::from_secs(2)).await;
    assert_eq!(time.elapsed().unwrap_or_default().as_secs(), 2);
}