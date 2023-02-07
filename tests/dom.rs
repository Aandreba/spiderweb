use std::time::Duration;
use futures::StreamExt;
use spiderweb::{
    dom::{Text, append_to_body},
    state::State,
    time::Interval,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn client_macro() -> Result<(), JsValue> {
    let text = State::new(String::new());
    let interval = Interval::new(
        || text.mutate(|x| x.push('a')),
        Duration::from_millis(500)
    );
    
    let text = Text::new_stringify(&text);
    append_to_body(text)?;
    interval.take(5).collect::<()>().await;

    return Ok(())
}
