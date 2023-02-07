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

#[inline(always)]
pub(crate) fn noop() {}
