#![warn(missing_docs)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
#[macro_use]
mod combo_vec;

#[cfg(feature = "alloc")]
pub use combo_vec::ComboVec;

#[macro_use]
mod re_arr;

pub use re_arr::ReArr;
