#![cfg_attr(feature = "nightly", feature(fn_traits, unboxed_closures, tuple_trait, trait_alias, downcast_unchecked, nonzero_ops, ptr_metadata, min_specialization))]
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

#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        $crate::eprintln!("[{}:{}]", $crate::file!(), $crate::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::eprintln!("[{}:{}] {} = {:#?}",
                    ::std::file!(), ::std::line!(), ::std::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    pub(crate) type WeakRef;

    #[wasm_bindgen(constructor, catch)]
    pub fn new(target: &wasm_bindgen::JsValue) -> Result<WeakRef, wasm_bindgen::JsValue>;

    #[wasm_bindgen(structural, method, js_name = deref)]
    fn _deref(this: &WeakRef) -> wasm_bindgen::JsValue;

    #[allow(unused_doc_comments)]
    #[doc(hidden)]
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &wasm_bindgen::JsValue);
    
    #[allow(unused_doc_comments)]
    #[doc(hidden)]
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &wasm_bindgen::JsValue);
}

impl WeakRef {
    #[inline]
    pub fn deref (&self) -> Option<wasm_bindgen::JsValue> {
        let inner = self._deref();
        if inner.is_undefined() { return None }
        return Some(inner)
    }
}

pub(crate) extern crate self as spiderweb;
#[doc(hidden)]
pub extern crate wasm_bindgen;
#[doc(hidden)]
pub extern crate js_sys;

pub mod sync;
/// Document Object Model
pub mod dom;
/// Cells designed to modify and propagate state.
pub mod state;
/// Task-related functionality
pub mod task;
/// Time-related functionality
pub mod time;
pub mod flag;

pub use spiderweb_proc::*;

#[inline(always)]
pub(crate) fn noop() {}