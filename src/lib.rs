#![warn(missing_docs)]

//! [`ComboVec`] is for creating a "combo stack array-heap vector", or simply a resizable array with a vector for extra allocations.
//!
//! Not only that, but this library has [`ReArr`] if you just want the resizable array part!
//!
//! Create a new [`ComboVec`] with the `combo_vec!` macro and a new [`ReArr`] with the [`re_arr!`] macro.
//!
//! This works by allocating an array of `T` on the stack, and then using a Vec on the heap for overflow.
//!
//! The stack-allocated array is always used to store the first `N` elements, even when the array is resized.
//!
//! _No_ `Default`, `Copy`, or `Clone` traits are required for `T` at all;
//! but if T does implement any of them, then [`ComboVec`] and [`ReArr`] will also implement them.
//! This also applied to `PartialEq`, `PartialOrd`, `Eq`, `Ord`, `Hash`, `Debug`, and `Display`.
//!
//! ## Why use [`ComboVec`]
//!
//! This is mostly used for when you know the maximum number of elements that will be stored 99% if the time,
//! but don't want to cause errors in the last 1% and also won't want to give up on the performance of using the stack instead of the heap most of the time.
//!
//! I've gotten performance bumps with [`ComboVec`] over the similar type `SmallVec` (both with and without it's `union` feature.)
//!
//! In a test of pushing 2048 (pre-allocated) elements, almost a 54% performance increase is shown:
//!
//! - [`ComboVec`]: 4.54 µs
//! - `SmallVec`: 9.33 µs
//!
//! The [`combo_vec!`] macro is very nice and convenient to use even in const contexts.
//!
//! ```rust
//! use combo_vec::{combo_vec, ComboVec};
//!
//! const SOME_ITEMS: ComboVec<i8, 3> = combo_vec![1, 2, 3];
//! const MANY_ITEMS: ComboVec<u16, 90> = combo_vec![5; 90];
//! const EXTRA_ITEMS: ComboVec<&str, 5> = combo_vec!["Hello", "world", "!"; None, None];
//!
//! // Infer the type and size of the ComboVec
//! const NO_STACK_F32: ComboVec<f32, 0> = combo_vec![];
//!
//! // No const-initialization is needed to create a ComboVec with allocated elements on the stack
//! use std::collections::HashMap;
//! const EMPTY_HASHMAP_ALLOC: ComboVec<HashMap<&str, i32>, 3> = combo_vec![];
//!
//! // Creating a new ComboVec at compile time and doing this does have performance benefits
//! let my_combo_vec = EMPTY_HASHMAP_ALLOC;
//! ```
//!
//! [`ComboVec`] also implements many methods that are exclusive to `Vec` such as `extend`, `truncate`, `push`, `join` etc.
//!
//! ## Why use [`ReArr`]
//!
//! In a test of pushing 2048 (pre-allocated) elements, it ties for performance with `ArrayVec`:
//!
//! - [`ReArr`]: 4.07 µs
//! - `ArrayVec`: 4.00 µs
//!
//! The [`re_arr!`] macro is very nice and convenient to use even in const contexts.
//!
//! ```rust
//! use combo_vec::{re_arr, ReArr};
//!
//! const SOME_ITEMS: ReArr<i8, 3> = re_arr![1, 2, 3];
//! const MANY_ITEMS: ReArr<u16, 90> = re_arr![5; 90];
//! const EXTRA_ITEMS: ReArr<&str, 5> = re_arr!["Hello", "world", "!"; None, None];
//!
//! // Infer the type and size of the ReArr
//! const NO_STACK_F32: ReArr<f32, 0> = re_arr![];
//!
//! // No const-initialization is needed to create a ComboVec with allocated elements on the stack
//! use std::collections::HashMap;
//! const EMPTY_HASHMAP_ALLOC: ReArr<HashMap<&str, i32>, 3> = re_arr![];
//!
//! // Creating a new ReArr at compile time and doing this does have performance benefits
//! let my_re_arr = EMPTY_HASHMAP_ALLOC;
//! ```
//!
//! `ReArr` also implements many methods that are exclusive to `Vec` such as `extend`, `truncate`, `push`, `join` etc.
//!
//! ## Examples
//!
//! A quick look at a basic example and some methods that are available:
//!
//! ```rust
//! use combo_vec::combo_vec;
//!
//! let mut combo_vec = combo_vec![1, 2, 3];
//! // Allocate an extra element on the heap
//! combo_vec.push(4);
//! // Truncate to a length of 2
//! combo_vec.truncate(2);
//! // Fill the last element on the stack, then allocate the next items on the heap
//! combo_vec.extend([3, 4, 5]);
//! ```
//!
//! ### Allocating empty memory on the stack
//!
//! You can allocate memory on the stack for later use without settings values to them!
//!
//! No Copy or Default traits required.
//!
//! ```rust
//! use combo_vec::{ComboVec, ReArr};
//!
//! // Allocate a new space to store 17 elements on the stack.
//! let empty_combo_vec = ComboVec::<f32, 17>::new();
//! let empty_re_arr = ReArr::<f32, 17>::new();
//! ```
//!
//! ### Allocating memory on the stack in const contexts
//!
//! The main benefit of using the [`combo_vec!`]/[`re_arr!`] macros is that everything it does can be used in const contexts.
//!
//! This allows you to allocate a [`ComboVec`] at the start of your program in a `Mutex` or `RwLock`, and have minimal runtime overhead.
//!
//! ```rust
//! use combo_vec::{combo_vec, ComboVec, re_arr, ReArr};
//!
//! // Create a global variable for the various program states for a semi-unspecified length
//! use std::{collections::HashMap, sync::RwLock};
//! static PROGRAM_STATES: RwLock<ComboVec<HashMap<String, i32>, 20>> = RwLock::new(combo_vec![]);
//!
//! // If we know the stack will never be larger than 20 elements,
//! // we can get a performance boost by using ReArr instead of ComboVec
//! let mut runtime_stack = ReArr::<i32, 20>::new();
//! ```
//!
//! ### Go fast with const & copy
//!
//! We can take advantage of [`ComboVec`] and [`ReArr`] by creating one const context then copying it to our runtime variable.
//! This is much faster than creating a new [`ComboVec`] at runtime, and `T` does _not_ need to be `Copy`.
//!
//! Here's a basic look at what this looks like:
//!
//! ```rust
//! use combo_vec::{combo_vec, ComboVec};
//!
//! const SOME_ITEMS: ComboVec<String, 2> = combo_vec![];
//!
//! for _ in 0..50 {
//!     let mut empty_combo_vec = SOME_ITEMS;
//!     empty_combo_vec.push("Hello".to_string());
//!     empty_combo_vec.push("world".to_string());
//!     println!("{}!", empty_combo_vec.join(" "));
//! }
//! ```

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
