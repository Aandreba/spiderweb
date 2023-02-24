use spiderweb::{dom::{element::Element, component::Component}, state::Writeable};
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
    
    let element = Component::new("div", Writeable::new(0i32));

    let plus = element.append_child(Element::new("button"))?;
    plus.add_event_listener("click", |x| *x += 1);

    let minus = element.append_child(Element::new("button"))?;
    minus.add_event_listener("click", |x| *x -= 1);

    return Ok(());
}
// TODO styles via tailwind
