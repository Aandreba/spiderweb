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

pub mod time;
pub mod channel;
pub mod task;

#[inline(always)]
pub(crate) fn noop () {}