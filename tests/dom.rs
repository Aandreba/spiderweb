use std::time::Duration;
use futures::StreamExt;
use spiderweb::{
    dom::{append_to_body, Text},
    state::State,
    time::Interval,
};
use spiderweb_proc::client;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn text() {
    // Static
    append_to_body("Hello world!").unwrap();

    // Dynamic
    let text = State::new(String::new());
    let interval = Interval::new(|| text.mutate(|x| x.push('a')), Duration::from_millis(500));

    let text = Text::new_stringify(&text);
    append_to_body(&text).unwrap();

    let _ = interval.take(5).collect::<Vec<_>>().await;
    drop(text);
}

#[wasm_bindgen_test]
async fn client_macro() -> Result<(), JsValue> {
    let text = State::new(String::new());
    let interval = Interval::new(
        || text.mutate(|x| x.push('a')),
        Duration::from_millis(500)
    );
    
    // todo fix
    let item = client! {
        <span>{&text}</span>
    }?;

    append_to_body(item)?;
    interval.take(5).collect::<()>().await;

    return Ok(())
}
