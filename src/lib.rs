#![warn(missing_docs, clippy::pedantic, clippy::nursery)]

//! An array that can be resized at runtime but allocated stack space at compile time and doesn't move any data off the stack when it overflows.
//!
//! Create a new [`ComboVec`] with the [`combo_vec!`] macro.
//!
//! This works by allocating an array of `T` on the stack, and then using a Vec on the heap for overflow.
//!
//! The stack-allocated array is always used to store the first `N` elements, even when the array is resized.
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
//! assert_eq!(combo_vec.len(), 4);
//! // Truncate to only the first 2 elements
//! combo_vec.truncate(2);
//! assert_eq!(combo_vec.len(), 2);
//! // Fill the last element on the stack, then allocate the next two items on the heap
//! combo_vec.extend([3, 4, 5]);
//!
//! // Many normal Vec methods are available (WIP)
//! assert_eq!(combo_vec.len(), 5);
//! assert_eq!(combo_vec[1], 2);
//! assert_eq!(combo_vec.last(), Some(&5));
//! assert_eq!(combo_vec.to_vec(), vec![1, 2, 3, 4, 5]);
//! assert_eq!(combo_vec.join(", "), "1, 2, 3, 4, 5");
//!
//! // The amount of elements currently stored on the stack
//! assert_eq!(combo_vec.stack_len(), 3);
//! // The amount of elements currently stored on the heap
//! assert_eq!(combo_vec.heap_len(), 2);
//!
//! // The total amount of elements that can be stored on the stack (3 elements were initially allocated)
//! assert_eq!(combo_vec.stack_capacity(), 3);
//! // The total amount of elements currently allocated on the heap - may vary depending on what Rust decides to do
//! assert!(combo_vec.heap_capacity() >= 2);
//! // The total number of elements currently allocated in memory (stack + heap)
//! assert!(combo_vec.capacity() >= 5);
//! ```
//!
//! ### Allocating empty memory on the stack
//!
//! You can allocate memory on the stack for later use without settings values to them!
//!
//! No Copy or Default traits required.
//!
//! ```rust
//! use combo_vec::combo_vec;
//!
//! // Easily allocate a new ComboVec where 16 elements can be stored on the stack.
//! let default_f32_vec = combo_vec![f32];
//!
//! // No const default implementation is needed to create a ComboVec with allocated elements on the stack
//! let empty_f32_vec = combo_vec![f32; 17];
//!
//! // An empty array of hashmaps (which can't be initialized in const contexts) can be allocated space on the stack.
//! use std::collections::HashMap;
//! let empty_hashmap_vec = combo_vec![HashMap<&str, i32>; 3];
//! ```
//!
//! ### Allocating memory on the stack in const contexts
//!
//! The main benefit of using the [`combo_vec!`] macro is that everything it does can be used in const contexts.
//!
//! This allows you to allocate a [`ComboVec`] at the start of your program in a `Mutex` or `RwLock`, and have minimal runtime overhead.
//!
//! ```rust
//! use combo_vec::{combo_vec, ComboVec};
//!
//! const SOME_ITEMS: ComboVec<i8, 3> = combo_vec![1, 2, 3];
//! const MANY_ITEMS: ComboVec<u16, 90> = combo_vec![5; 90];
//!
//! // Infer the type and size of the ComboVec
//! const INFER_F32: ComboVec<f32, 13> = combo_vec![];
//!
//! // No const-initialization is needed to create a ComboVec with allocated elements on the stack
//! use std::collections::HashMap;
//! const EMPTY_HASHMAP_ALLOC: ComboVec<HashMap<&str, i32>, 3> = combo_vec![];
//!
//! /// Create a global-state RwLock that can store a ComboVec
//! use std::sync::RwLock;
//! static PROGRAM_STATE: RwLock<ComboVec<&str, 20>> = RwLock::new(combo_vec![]);
//! ```
//!
//! ### Go fast with const & copy
//!
//! Making an entire, new `ComboVec` at runtime can be slower than just allocating a new array or a new vector - because it needs to do both.
//!
//! We can take advantage of `ComboVec` being a `Copy` type, and use it to create a new `ComboVec` in a const context then copy it to our runtime variable. This is much faster than creating a new `ComboVec` at runtime.
//!
//! Here's a basic look at what this looks like:
//!
//! ```rust
//! use combo_vec::{combo_vec, ComboVec};
//!
//! const SOME_ITEMS: ComboVec<String, 2> = combo_vec![];
//!
//! for _ in 0..5 {
//!     let mut empty_combo_vec = SOME_ITEMS;
//!     empty_combo_vec.push("Hello".to_string());
//!     empty_combo_vec.push("World".to_string());
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
