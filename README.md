# combo_vec

[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/) 

"combo_vec" is a library for creating a "combo stack array-heap vector", or simply a resizable array.

Create a new `ReArr` with the `rearr!` macro.

This works by allocating an array of `T` on the stack, and then using a Vec on the heap for overflow.

The stack-allocated array is always used to store the first `N` elements, even when the array is resized.

## Examples

A quick look at a basic example and some methods that are available:

```rust
use combo_vec::{rearr, ReArr};

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
use combo_vec::{rearr, ReArr};

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
const NO_STACK_F32: ReArr<f32, 0> = rearr![];

// No const-initialization is needed to create a ReArr with allocated elements on the stack
use std::collections::HashMap;
const EMPTY_HASHMAP_ALLOC: ReArr<HashMap<&str, i32>, 3> = rearr![];

/// Create a global-state RwLock that can store a ReArr 
use std::sync::RwLock;
static PROGRAM_STATE: RwLock<ReArr<&str, 20>> = RwLock::new(rearr![]);
```