#![warn(missing_docs, clippy::pedantic, clippy::nursery)]

//! An array that can be resized at runtime but allocated stack space at compile time and doesn't move any data off the stack when it overflows.
//!
//! Create a new [`ReArr`] with the [`rearr!`] macro.
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
//! use combo_vec::rearr;
//!
//! let mut resizeable_vec = rearr![1, 2, 3];
//! // Allocate an extra element on the heap
//! resizeable_vec.push(4);
//! assert_eq!(resizeable_vec.len(), 4);
//! // Truncate to only the first 2 elements
//! resizeable_vec.truncate(2);
//! assert_eq!(resizeable_vec.len(), 2);
//! // Fill the last element on the stack, then allocate the next two items on the heap
//! resizeable_vec.extend([3, 4, 5]);
//!
//! // Many normal Vec methods are available (WIP)
//! assert_eq!(resizeable_vec.len(), 5);
//! assert_eq!(resizeable_vec[1], 2);
//! assert_eq!(resizeable_vec.last(), Some(&5));
//! assert_eq!(resizeable_vec.to_vec(), vec![1, 2, 3, 4, 5]);
//! assert_eq!(resizeable_vec.join(", "), "1, 2, 3, 4, 5");
//!
//! // The amount of elements currently stored on the stack
//! assert_eq!(resizeable_vec.stack_len(), 3);
//! // The amount of elements currently stored on the heap
//! assert_eq!(resizeable_vec.heap_len(), 2);
//!
//! // The total amount of elements that can be stored on the stack (3 elements were initially allocated)
//! assert_eq!(resizeable_vec.stack_capacity(), 3);
//! // The total amount of elements currently allocated on the heap - may vary depending on what Rust decides to do
//! assert!(resizeable_vec.heap_capacity() >= 2);
//! // The total number of elements currently allocated in memory (stack + heap)
//! assert!(resizeable_vec.capacity() >= 5);
//! ```
//!
//! ### Allocating empty memory on the stack
//!
//! You can allocate memory on the stack for later use without settings values to them!
//!
//! No Copy or Default traits required.
//!
//! ```rust
//! use combo_vec::rearr;
//!
//! // Easily allocate a new ReArr where 16 elements can be stored on the stack.
//! let default_f32_vec = rearr![f32];
//!
//! // No const default implementation is needed to create a ReArr with allocated elements on the stack
//! let empty_f32_vec = rearr![f32; 17];
//!
//! // An empty array of hashmaps (which can't be initialized in const contexts) can be allocated space on the stack.
//! use std::collections::HashMap;
//! let empty_hashmap_vec = rearr![HashMap<&str, i32>; 3];
//! ```
//!
//! ### Allocating memory on the stack in const contexts
//!
//! The main benefit of using the [`rearr!`] macro is that everything it does can be used in const contexts.
//!
//! This allows you to allocate a [`ReArr`] at the start of your program in a `Mutex` or `RwLock`, and have minimal runtime overhead.
//!
//! ```rust
//! use combo_vec::{rearr, ReArr};
//!
//! const SOME_ITEMS: ReArr<i8, 3> = rearr![1, 2, 3];
//! const MANY_ITEMS: ReArr<u16, 90> = rearr![5; 90];
//!
//! // Infer the type and size of the ReArr
//! const INFER_F32: ReArr<f32, 13> = rearr![];
//!
//! // No const-initialization is needed to create a ReArr with allocated elements on the stack
//! use std::collections::HashMap;
//! const EMPTY_HASHMAP_ALLOC: ReArr<HashMap<&str, i32>, 3> = rearr![];
//!
//! /// Create a global-state RwLock that can store a ReArr
//! use std::sync::RwLock;
//! static PROGRAM_STATE: RwLock<ReArr<&str, 20>> = RwLock::new(rearr![]);
//! ```
//!
//! ### Go fast with const & copy
//!
//! Making an entire, new `ReArr` at runtime can be slower than just allocating a new array or a new vector - because it needs to do both.
//!
//! We can take advantage of `ReArr` being a `Copy` type, and use it to create a new `ReArr` in a const context then copy it to our runtime variable. This is much faster than creating a new `ReArr` at runtime.
//!
//! Here's a basic look at what this looks like:
//!
//! ```rust
//! use combo_vec::{rearr, ReArr};
//!
//! const SOME_ITEMS: ReArr<String, 2> = rearr![];
//!
//! for _ in 0..5 {
//!     let mut empty_rearr = SOME_ITEMS;
//!     empty_rearr.push("Hello".to_string());
//!     empty_rearr.push("World".to_string());
//!     println!("{}!", empty_rearr.join(" "));
//! }
//! ```

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
#[macro_use]
mod combo_vec;

#[cfg(feature = "alloc")]
pub use combo_vec::ReArr;
