use futures::StreamExt;
use spiderweb::{
    dom::{
        append_to_body,
        view::{Alignment, Orientation, Pane},
        Text,
    },
    state::StateCell,
    time::{Interval},
};
use spiderweb_proc::client;
use std::{
    time::Duration,
};
use wasm_bindgen::{JsValue};
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

/*
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
*/

#[wasm_bindgen_test]
async fn pane() -> Result<(), JsValue> {
    let pane = Pane::new(
        Orientation::Horizontal,
        Alignment::Center,
        Alignment::Center,
    )?;
    pane.push(client! { <span>{"Hello"}</span> }, 1.0)?;
    pane.push(client! { <span>{"world"}</span> }, 1.0)?;

    let _ = append_to_body(pane);
    return Ok(());
}

// TODO styles via tailwind
