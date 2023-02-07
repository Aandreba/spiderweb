use futures::StreamExt;
use spiderweb::{
    dom::{append_to_body, std::Button, Text},
    state::State,
    time::Interval,
};
use spiderweb_proc::client;
use std::{
    ops::{AddAssign, SubAssign},
    rc::Rc,
    time::Duration,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::{console_log, wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn client_macro() -> Result<(), JsValue> {
    let text = State::new(String::new());
    let interval = Interval::new(|| text.update(|x| x.push('a')), Duration::from_millis(500));

    let text = client! {
        <span>
            <i>{"Hello, "}</i>
            <b>{Text::display(&text)}</b>
        </span>
    }?;

    append_to_body(text)?;
    interval.take(5).collect::<()>().await;

    return Ok(());
}

#[wasm_bindgen_test]
async fn counter() -> Result<(), JsValue> {
    let value = Rc::new(State::new(0i32));

    let my_value = value.clone();
    let inc = Button::new("+", move || {
        my_value.update(|x| x.add_assign(1))
    });

    let my_value = value.clone();
    let dec = Button::new("-", move || my_value.update(|x| x.sub_assign(1)));

    let text = client! {
        <div>
            <span>{"Current value: "}{Text::display(&value)}</span>
            <br/>
            {inc}
            {dec}
        </div>
    }?;

    append_to_body(text)?;

    return Ok(());
}
