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

/// Time-related functionality
pub mod time;
/// `!Send` and `!Sync` channels designed to send information between JavaScript contexts
pub mod channel;
/// Task-related functionality
pub mod task;

#[inline(always)]
pub(crate) fn noop () {}