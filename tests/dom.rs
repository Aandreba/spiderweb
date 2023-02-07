use std::time::Duration;
use futures::StreamExt;
use spiderweb::{
    dom::{Text, append_to_body},
    state::State,
    time::Interval,
};
use spiderweb_proc::client;
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
    
    let text = client! {
        <span>
            <i>{"Hello, "}</i>
            <b>{Text::display(&text)}</b>
        </span>
    }?;

    append_to_body(text)?;
    interval.take(5).collect::<()>().await;

    return Ok(())
}
