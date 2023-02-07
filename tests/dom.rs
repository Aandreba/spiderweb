use std::time::Duration;
use futures::StreamExt;
use spiderweb::{
    dom::{append_to_body, IntoComponent},
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
    
    // todo fix
    let text = IntoComponent::into_component(&text);
    let item = client! {
        <span>{&text}</span>
    }?;

    append_to_body(item)?;
    interval.take(5).collect::<()>().await;
    drop(text);

    return Ok(())
}
