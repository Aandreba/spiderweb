use futures::StreamExt;
use js_sys::Array;
use spiderweb::{
    dom::{append_to_body, Text},
    state::State,
    time::{Interval, Timeout},
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
        Duration::from_secs(1),
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
async fn text() {
    // Static
    append_to_body(&Text::new_static("Hello")).unwrap();

    // Dynamic
    let text = State::new(String::new());
    let interval = Interval::new(|| text.mutate(|x| x.push('a')), Duration::from_millis(500));

    let text = Text::new_stringify(&text);
    append_to_body(&text).unwrap();

    let _ = interval.take(5).collect::<Vec<_>>().await;
    drop(text);
}
