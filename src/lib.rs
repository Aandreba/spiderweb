#![cfg_attr(feature = "nightly", feature(fn_traits, unboxed_closures, tuple_trait, nonzero_ops, min_specialization))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(all(target_family = "wasm", not(target_feature = "atomics"))))]
compile_error!("Unsupported target");

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    };
}

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[allow(unused_doc_comments)]
    #[doc(hidden)]
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &wasm_bindgen::JsValue);
    
    #[allow(unused_doc_comments)]
    #[doc(hidden)]
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &wasm_bindgen::JsValue);
}

pub(crate) extern crate self as spiderweb;
#[doc(hidden)]
pub extern crate wasm_bindgen;
#[doc(hidden)]
pub extern crate js_sys;

/// `!Send` and `!Sync` channels designed to send information between JavaScript contexts
pub mod channel;
/// Document Object Model
pub mod dom;
/// Cells designed to modify and propagate state.
pub mod state;
/// Task-related functionality
pub mod task;
/// Time-related functionality
pub mod time;
pub extern crate spiderweb_proc as macros;

#[inline(always)]
pub(crate) fn noop() {}