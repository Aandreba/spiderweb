use crate::console::Console;
use parse::Element;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

mod console;
mod parse;

#[proc_macro]
pub fn client(items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let element = parse_macro_input!(items as Element);
    return element.to_token_stream().into();
}

#[proc_macro]
pub fn println(items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    return match parse_macro_input!(items as Console) {
        Console::Format(x) => {
            quote! {
                ::spiderweb::log(::spiderweb::wasm_bindgen::JsValue::from_str(&::std::format!(#x)))
            }
        },
        Console::Value(x) if x.len() == 1 => {
            quote! {
                ::spiderweb::log(::std::convert::AsRef::<::spiderweb::wasm_bindgen::JsValue>::as_ref(#x))
            }
        }
        Console::Value(x) => {
            quote! {
                ::spiderweb::log(&<::spiderweb::js_sys::Array as ::std::iter::FromIterator<_>>::from_iter([#x]))
            }
        }
    }
    .into();
}

#[proc_macro]
pub fn eprintln(items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    return match parse_macro_input!(items as Console) {
        Console::Format(x) => {
            quote! {
                ::spiderweb::error(::spiderweb::wasm_bindgen::JsValue::from_str(&::std::format!(#x)))
            }
        },
        Console::Value(x) if x.len() == 1 => {
            quote! {
                ::spiderweb::error(::std::convert::AsRef::<::spiderweb::wasm_bindgen::JsValue>::as_ref(#x))
            }
        }
        Console::Value(x) => {
            quote! {
                ::spiderweb::error(&<::spiderweb::js_sys::Array as ::std::iter::FromIterator<_>>::from_iter([#x]))
            }
        }
    }
    .into();
}
