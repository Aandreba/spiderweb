use std::time::Duration;
use futures::StreamExt;
use safeweb::time::{Interval, Timeout};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn interval () {
    let mut secs = 0;
    let int = Interval::new(|| {
        secs += 1;
        secs
    }, Duration::from_secs(1));
    
    let mut int = int.take(5).enumerate();
    while let Some((i, x)) = int.next().await {
        assert_eq!(i + 1, x);
    }
}

#[wasm_bindgen_test]
async fn timeout () {
    let mut secs = time;
    let int = Timeout::new(|| todo!(), Duration::from_secs(1));
    
    assert_eq!()
}