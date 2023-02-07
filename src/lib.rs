#![cfg_attr(feature = "nightly", feature(fn_traits, unboxed_closures, tuple_trait))]
#![cfg_attr(docsrs, feature(doc_cfg))]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    };
}

#[doc(hidden)]
pub extern crate wasm_bindgen;

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
