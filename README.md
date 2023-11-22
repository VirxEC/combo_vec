# combo_vec

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/) 

[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)

"combo_vec" is a library for creating a "combo stack array-heap vector", or simply a resizable array.

Create a new `ReArr` with the `rearr!` macro.

This works by allocating an array of `T` on the stack, and then using a Vec on the heap for overflow.

The stack-allocated array is always used to store the first `N` elements, even when the array is resized.

## Why use combo_vec

This is mostly used for when you know the maximum number of elements that will be stored 99% if the time, but don't want to cause errors in the last 1% and also won't want to give up on the performance of using the stack instead of the heap most of the time.

`ReArr` also implemented many methods that are exclusive to `Vec` such as `extend`, `truncate`, `push`, `join` etc.

In the real world I've seen a performance increase by using `ReArr` over `Vec` on memory-bandwidth limited devices in situations where the `Vec` is being pushed and popped a lot from. I found this performance increase and use `ReArr` in the `rl_ball_sym` crate for this performance bump.

## Examples

A quick look at a basic example and some methods that are available:

```rust
use combo_vec::rearr;

let mut resizeable_vec = rearr![1, 2, 3];
// Allocate an extra element on the heap
resizeable_vec.push(4);
// Truncate to only the first 2 elements
resizeable_vec.truncate(2);
// Fill the last element on the stack, then allocate the next two items on the heap
resizeable_vec.extend([3, 4, 5]);
```

### Allocating empty memory on the stack

You can allocate memory on the stack for later use without settings values to them!

No Copy or Default traits required.

```rust
use combo_vec::rearr;

// Easily allocate a new ReArr where 16 elements can be stored on the stack.
let default_f32_vec = rearr![f32];

// Allocate a new, empty ReArr with 17 elements abled to be stored on the stack.
let empty_f32_vec = rearr![f32; 17];
```

### Allocating memory on the stack in const contexts

The main benefit of using the `rearr!` macro is that everything it does can be used in const contexts.

This allows you to allocate a ReArr at the start of your program in a Mutex or RwLock, and have minimal runtime overhead.

```rust
use combo_vec::{rearr, ReArr};

const SOME_ITEMS: ReArr<i8, 3> = rearr![1, 2, 3];
const MANY_ITEMS: ReArr<u16, 90> = rearr![5; 90];

// Infer the type and size of the ReArr
const NO_STACK_F32: ReArr<f32, 13> = rearr![];

// No const default implementation is needed to create a ReArr with allocated elements on the stack
use std::collections::HashMap;
const EMPTY_HASHMAP_ALLOC: ReArr<HashMap<&str, i32>, 3> = rearr![];

/// Create a global-state RwLock that can store a ReArr 
use std::sync::RwLock;
static PROGRAM_STATE: RwLock<ReArr<&str, 20>> = RwLock::new(rearr![]);
```

### Go fast with const & copy

Making an entire, new `ReArr` at runtime can be slower than just allocating a new array or a new vector - because it needs to do both.

We can take advantage of `ReArr` being a `Copy` type, and use it to create a new `ReArr` in a const context then copy it to our runtime variable. This is much faster than creating a new `ReArr` at runtime. `T` does _not_ need to be `Copy`.

Here's a basic look at what this looks like:

```rust
use combo_vec::{rearr, ReArr};

const SOME_ITEMS: ReArr<String, 2> = rearr![];

for _ in 0..5 {
    let mut empty_rearr = SOME_ITEMS;
    empty_rearr.push("Hello".to_string());
    empty_rearr.push("World".to_string());
    println!("{}!", empty_rearr.join(" "));
}
```
