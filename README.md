# combo_vec

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)

"combo_vec" is a library for creating a "combo stack array-heap vector", or simply a resizable array with a vector for extra allocations.

Not only that, but this library has `ReArr` if you just want the resizable array part (which can be faster than even SmallVec.)

Create a new `ComboVec` with the `combo_vec!` macro and a new `ReArr` with the `re_arr!` macro.

This works by allocating an array of `T` on the stack, and then using a Vec on the heap for overflow.

The stack-allocated array is always used to store the first `N` elements, even when the array is resized.

## Why use ComboVec

This is mostly used for when you know the maximum number of elements that will be stored 99% if the time, but don't want to cause errors in the last 1% and also won't want to give up on the performance of using the stack instead of the heap most of the time.

In a test of pushing 2048 (pre-allocated) elements, almost a 10% performance increase is shown:

- `ReArr`: 7.32 µs
- `SmallVec`: 8.15 µs

`ComboVec` also implements many methods that are exclusive to `Vec` such as `extend`, `truncate`, `push`, `join` etc.

## Why use ReArr

I've gotten performance bumps with `ReArr` over the similar type SmallVec (both with and without it's `union` feature.)

In a test of pushing 2048 (pre-allocated) elements, almost a 38% performance increase is shown:

- `ReArr`: 5.00 µs
- `SmallVec`: 8.15 µs

`ReArr` also implements many methods that are exclusive to `Vec` such as `extend`, `truncate`, `push`, `join` etc.

## Examples

A quick look at a basic example and some methods that are available:

```rust
use combo_vec::combo_vec;

let mut combo_vec = combo_vec![1, 2, 3];
// Allocate an extra element on the heap
combo_vec.push(4);
// Truncate to only the first element
combo_vec.truncate(2);
// Fill the last elements on the stack, then allocate the next item on the heap
combo_vec.extend([3, 4, 5]);
```

### Allocating empty memory on the stack

You can allocate memory on the stack for later use without settings values to them!

No Copy or Default traits required.

```rust
use combo_vec::combo_vec;

// Easily allocate a new ComboVec where 16 elements can be stored on the stack.
let default_f32_vec = combo_vec![f32];

// Allocate a new, empty ComboVec with space to store 17 elements on the stack.
let empty_f32_vec = combo_vec![f32; 17];
```

### Allocating memory on the stack in const contexts

The main benefit of using the `combo_vec!` macro is that everything it does can be used in const contexts.

This allows you to allocate a ComboVec at the start of your program in a Mutex or RwLock, and have minimal runtime overhead.

```rust
use combo_vec::{combo_vec, ComboVec};

const SOME_ITEMS: ComboVec<i8, 3> = combo_vec![1, 2, 3];
const MANY_ITEMS: ComboVec<u16, 90> = combo_vec![5; 90];

// Infer the type and size of the ComboVec
const NO_STACK_F32: ComboVec<f32, 13> = combo_vec![];

// No const default implementation is needed to create a ComboVec with allocated elements on the stack
use std::collections::HashMap;
const EMPTY_HASHMAP_ALLOC: ComboVec<HashMap<&str, i32>, 3> = combo_vec![];

/// Create a global-state RwLock that can store a ComboVec
use std::sync::RwLock;
static PROGRAM_STATE: RwLock<ComboVec<&str, 20>> = RwLock::new(combo_vec![]);
```

### Go fast with const & copy

Making an entire, new `ComboVec` at runtime can be slower than just allocating a new array or a new vector - because it needs to do both.

We can take advantage of `ComboVec` being a `Copy` type, and use it to create a new `ComboVec` in a const context then copy it to our runtime variable. This is much faster than creating a new `ComboVec` at runtime. `T` does _not_ need to be `Copy`.

Here's a basic look at what this looks like:

```rust
use combo_vec::{combo_vec, ComboVec};

const SOME_ITEMS: ComboVec<String, 2> = combo_vec![];

for _ in 0..5 {
    let mut empty_combo_vec = SOME_ITEMS;
    empty_combo_vec.push("Hello".to_string());
    empty_combo_vec.push("World".to_string());
    println!("{}!", empty_combo_vec.join(" "));
}
```
