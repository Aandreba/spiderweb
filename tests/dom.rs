use core::num;
use futures::StreamExt;
use spiderweb::{
    dom::{
        append_to_body,
        view::{Alignment, Span},
        Component, Element,
    },
    state::State,
    time::{Interval, Timeout},
};
use spiderweb_proc::client;
use std::{any::Any, ops::Deref, rc::Rc, time::Duration};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

// #[wasm_bindgen_test]
// async fn client_macro() -> Result<(), JsValue> {
//     let text = StateCell::new(String::new());
// 
//     let mut name = "Alex".chars();
//     let interval = Interval::new(
//         || {
//             if let Some(c) = name.next() {
//                 text.update(|x| x.push(c))
//             }
//         },
//         Duration::from_millis(500),
//     );
// 
//     let text = client! {
//         <span>
//             <i>{"Hello, "}</i>
//             <b>{&text}</b>
//         </span>
//     }?;
// 
//     append_to_body(text)?;
//     interval.take(5).collect::<()>().await;
// 
//     return Ok(());
// }

#[wasm_bindgen_test]
async fn counter() -> Result<(), JsValue> {
    let count = State::new(0u32);

    let button = client! {
        <button on:click={move |_| count.add_assign(1)}>
            {"Clicked "}
            {Span::dynamic(&count, |count| match count {
                1 => "1 time".into(),
                other => format!("{other} times")
            })}
        </button>
    }?;

    append_to_body(button)?;
    return Ok(());
}

#[wasm_bindgen_test]
async fn numbers() -> Result<(), JsValue> {
    let numbers = State::new(vec![1, 2, 3, 4]);
    let sum = numbers.map_shared(|x| x.iter().sum::<usize>());

    let button = client! {
        <button on:click={move |_| {
            numbers.update(|x| x.push(x.len() + 1));
            numbers.with(|x| spiderweb::println!("{x:?}"));
        }}>
            {""}{" = "}{&*sum}
        </button>
    }?;

    append_to_body(button)?;
    return Ok(());
}

// TODO styles via tailwind
