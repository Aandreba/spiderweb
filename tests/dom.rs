use futures::StreamExt;
use spiderweb::{
    dom::{append_to_body, view::Button, Text},
    state::StateCell,
    task::sleep,
    time::{Interval, Timeout},
};
use spiderweb_proc::client;
use std::{
    ops::{AddAssign, SubAssign},
    rc::Rc,
    time::Duration,
};
use wasm_bindgen::{prelude::Closure, JsValue};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn client_macro() -> Result<(), JsValue> {
    let text = StateCell::new(String::new());
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
    let value = Rc::new(StateCell::new(0i32));

    let my_value = value.clone();
    let reset = Button::new("Reset", move || my_value.set(0));

    let my_value = value.clone();
    let inc = Button::new("+", move || my_value.update(|x| x.add_assign(1)));

    let my_value = value.clone();
    let dec = Button::new("-", move || my_value.update(|x| x.sub_assign(1)));

    let text = client! {
        <div>
            <span>
                {"Current value: "}
                <b>{Text::display(&value)}</b>
            </span>
            <br/>
            {inc}
            {dec}
            {reset}
        </div>
    }?;

    let handle = append_to_body(text)?;
    let text = Timeout::new(|| handle.detach(), Duration::from_secs_f32(2.5)).await;

    return Ok(());
}

// TODO styles via tailwind
