#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! An array that can be resized at runtime, without moving any data off the stack.
//!
//! Create a new [`ReArr`] with the [`rearr!`] macro.
//!
//! This works by allocating an array of `T` on the stack, and then using a Vec on the heap for overflow.
//!
//! The stack-allocated array is always used to store the first `N` elements, even when the array is resized.
//!
//! This is very useful when you normally have a small amount of items where storing them on the stack is ideal, but there is no actual upper bound to the amount of items you might need to handle.
//!
//! ## Examples
//!
//! A quick look at a basic example and some methods that are available:
//!
//! ```rust
//! use combo_vec::{rearr, ReArr};
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
//!
//! // The amount of elements currently stored on the stack
//! assert_eq!(resizeable_vec.stack_len(), 3);
//! // The amount of elements currently stored on the heap
//! assert_eq!(resizeable_vec.heap_len(), 2);
//!
//! // The total amount of elements that can be stored on the stack (3 elements were initially allocated)
//! assert_eq!(resizeable_vec.stack_capacity(), 3);
//! // The total amount of elements currently allocated on the heap - may vary depending on what Rust decides to do
//! resizeable_vec.heap_capacity();
//! // The total number of elements currently allocated in memory (stack + heap)
//! resizeable_vec.capacity();
//! ```
//!
//! ### Allocating empty memory on the stack
//!
//! You can allocate memory on the stack for later use without settings values to them!
//!
//! No Copy or Default traits required.
//!
//! ```rust
//! use combo_vec::{rearr, ReArr};
//!
//! // Allocate a new, empty ReArr where no elements can be stored on the stack.
//! // Not sure why you'd want to do this, but it's possible.
//! let no_stack_f32_vec = rearr![f32];
//!
//! // Allocate a new, empty ReArr with 17 elements abled to be stored on the stack.
//! let empty_f32_vec = rearr![f32; 17];
//! ```
//!
//! ### Allocating memory on the stack in const contexts
//!
//! The main benefit of using the [`rearr!`] macro is that everything it does can be used in const contexts.
//!
//! This allows you to allocate a ReArr at the start of your program in a Mutex or RwLock, and have minimal runtime overhead.
//!
//! ```rust
//! use combo_vec::{rearr, ReArr};
//!
//! const SOME_ITEMS: ReArr<i8, 3> = rearr![1, 2, 3];
//! const MANY_ITEMS: ReArr<u16, 90> = rearr![5; 90];
//! const NO_STACK_F32: ReArr<f32, 0> = rearr![f32];
//!
//! /// An empty array of hashmaps (which can't be initialized in const contexts) can be allocated space on the stack.
//! use std::collections::HashMap;
//! const HASHMAP_ALLOC: ReArr<HashMap<&str, i32>, 3> = rearr![HashMap<&str, i32>; 3];
//! ```
//!

use std::{
    array::IntoIter as ArrayIter,
    fmt::{Debug, Display, Write},
    hash::{Hash, Hasher},
    iter::{Chain, Flatten},
    vec::IntoIter as VecIter,
};

/// Easy creation of a new [`ReArr`].
///
/// ## Examples
///
/// ```rust
/// use combo_vec::{rearr, ReArr};
///
/// const SOME_ITEMS: ReArr<i8, 3> = rearr![1, 2, 3];
/// const MANY_ITEMS: ReArr<u16, 90> = rearr![5; 90];
/// const NO_STACK_F32: ReArr<f32, 0> = rearr![f32];
///
/// /// An empty array of hashmaps (which can't be initialized in const contexts) can be allocated space on the stack.
/// use std::collections::HashMap;
/// const HASHMAP_ALLOC: ReArr<HashMap<&str, i32>, 3> = rearr![HashMap<&str, i32>; 3];
///
/// let my_rearr = rearr![1, 2, 3];
/// ```
#[macro_export]
macro_rules! rearr {
    ($type:ty) => (
        ReArr::<$type, 0>::new()
    );
    ($type:ty; $n:literal) => (
        ReArr::<$type, $n>::new()
    );
    ($elem:expr; $n:expr) => (
        ReArr::from_arr([Some($elem); $n])
    );
    ($($x:expr),+ $(,)?) => (
        ReArr::from_arr([$(Some($x)),+])
    );
}

/// An array that can be resized at runtime, without moving any data off the stack.
///
/// Create a new [`ReArr`] using the [`rearr!`] macro.
///
/// ## Examples
///
/// ```rust
/// use combo_vec::{rearr, ReArr};
///
/// const SOME_ITEMS: ReArr<i8, 3> = rearr![1, 2, 3];
/// const MANY_ITEMS: ReArr<u16, 90> = rearr![5; 90];
/// const NO_STACK_F32: ReArr<f32, 0> = rearr![f32];
///
/// /// An empty array of hashmaps (which can't be initialized in const contexts) can be allocated space on the stack.
/// use std::collections::HashMap;
/// const HASHMAP_ALLOC: ReArr<HashMap<&str, i32>, 3> = rearr![HashMap<&str, i32>; 3];
///
/// let mut my_rearr = rearr![1, 2, 3];
/// // Allocate an extra element on the heap
/// my_rearr.push(4);
/// assert_eq!(my_rearr.len(), 4);
/// // Truncate to only the first 2 elements
/// my_rearr.truncate(2);
/// assert_eq!(my_rearr.len(), 2);
/// // Fill the last element on the stack, then allocate the next two items on the heap
/// my_rearr.extend([3, 4, 5]);
/// ```
#[derive(Clone, Debug)]
pub struct ReArr<T, const N: usize> {
    arr: [Option<T>; N],
    vec: Vec<T>,
}

impl<T: PartialEq, const N: usize> PartialEq for ReArr<T, N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T: PartialEq + Eq, const N: usize> Eq for ReArr<T, N> {}

impl<T: Hash, const N: usize> Hash for ReArr<T, N> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter().for_each(|x| x.hash(state));
    }
}

impl<T: Default, const N: usize> Default for ReArr<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> ReArr<T, N> {
    const DEFAULT_ARR_VALUE: Option<T> = None;

    /// Create a new, empty ReArr with with the ability for `N` element to be allocated on the stack.
    ///
    /// This is used by the [`rearr!`] macro, and you should consider using it instead.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{rearr, ReArr};
    ///
    /// let my_rearr = ReArr::<i32, 3>::new();
    /// let convient_rearr = rearr![i32; 3];
    /// assert_eq!(my_rearr, convient_rearr);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            arr: [Self::DEFAULT_ARR_VALUE; N],
            vec: Vec::new(),
        }
    }

    /// Allocate more memory to what can be stored on the heap.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
    }

    /// Create a resizable array from a fixed size array.
    ///
    /// This is used by the [`rearr!`] macro, and you should consider using it instead.
    ///
    /// ```rust
    /// use combo_vec::{rearr, ReArr};
    ///
    /// let my_rearr = ReArr::from_arr([Some(1), Some(2), Some(3)]);
    /// let convient_rearr = rearr![1, 2, 3];
    /// assert_eq!(my_rearr, convient_rearr);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_arr(arr: [Option<T>; N]) -> Self {
        Self { arr, vec: Vec::new() }
    }

    /// Push an element to the end of the array.
    ///
    /// If the array is full, the element will be pushed to the heap.
    #[inline]
    pub fn push(&mut self, val: T) {
        let stack_len = self.stack_len();
        if stack_len < N {
            self.arr[stack_len] = Some(val);
        } else {
            self.vec.push(val);
        }
    }

    /// Get any element from the array as a reference, returning `None` if out of bounds.
    #[must_use]
    #[inline]
    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < N {
            self.arr[idx].as_ref()
        } else {
            self.vec.get(idx - N)
        }
    }

    /// Get any element from the array as a mutable reference, `None` if out of bounds.
    #[must_use]
    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < N {
            self.arr[idx].as_mut()
        } else {
            self.vec.get_mut(idx - N)
        }
    }

    /// Whether or not where are any elements allocated on the heap instead of the stack
    #[inline]
    pub fn spilled(&self) -> bool {
        self.heap_len() > 0
    }

    /// How many elements are currently stored on the stack.
    #[inline]
    pub fn stack_len(&self) -> usize {
        self.arr.iter().flatten().count()
    }

    /// How many elements are currently stored on the heap.
    #[inline]
    pub fn heap_len(&self) -> usize {
        self.vec.len()
    }

    /// How many elements are currently stored.
    #[inline]
    pub fn len(&self) -> usize {
        self.stack_len() + self.heap_len()
    }

    /// How many elements can be stored on the stack.
    #[inline]
    pub const fn stack_capacity(&self) -> usize {
        N
    }

    /// How many elements can be stored on the currently allocated heap.
    #[inline]
    pub fn heap_capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// How many elements can be stored without reallocating anything.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.stack_capacity() + self.heap_capacity()
    }

    /// Reduce the number of elements to the given length.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len > self.len() {
            // do nothing
        } else if len >= N {
            self.vec.truncate(len - N);
        } else {
            self.arr[len..].iter_mut().for_each(|x| *x = None);
            self.vec.clear();
        }
    }

    /// Remove all elements from the array.
    #[inline]
    pub fn clear(&mut self) {
        self.arr.iter_mut().for_each(|x| *x = None);
        self.vec.clear();
    }

    /// Get the first element, returning `None` if there are no elements.
    #[inline]
    pub fn first(&self) -> Option<&T> {
        if N == 0 {
            self.vec.first()
        } else {
            self.arr[0].as_ref().or_else(|| self.vec.first())
        }
    }

    /// Get the last element, returning `None` if there are no elements.
    #[inline]
    pub fn last(&self) -> Option<&T> {
        self.vec.last().or_else(|| self.arr[N - 1].as_ref())
    }

    /// Check if there are no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an iterator over the elements of the array.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.arr.iter().filter_map(Option::as_ref).chain(self.vec.iter())
    }

    /// Get an iterator over the elements of the array, returning mutable references.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.arr.iter_mut().filter_map(Option::as_mut).chain(self.vec.iter_mut())
    }

    /// Extend this array with all the elements from the given iterator.
    #[inline]
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|x| self.push(x));
    }

    /// Get this [`ReArr`] represented as a [`Vec`], borrowing data instead of cloning it.
    #[inline]
    pub fn as_vec(&self) -> Vec<&T> {
        self.iter().collect()
    }
}

impl<T: Clone, const N: usize> ReArr<T, N> {
    /// Get this [`ReArr`] represented as a [`Vec`].
    /// 
    /// [`Vec`]: std::vec::Vec
    #[inline]
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

    /// Resizes the [`ReArr`] in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the [`ReArr`] is extended by the
    /// difference, with each additional slot filled with `val`.
    #[inline]
    pub fn resize(&mut self, new_len: usize, val: T) {
        if new_len >= N {
            self.vec.resize(new_len - N, val);
        } else {
            self.arr[..new_len].iter_mut().for_each(|x| *x = Some(val.clone()));
            self.arr[new_len..].iter_mut().for_each(|x| *x = None);
            self.vec = Vec::new();
        }
    }
}

impl<T, const N: usize> std::ops::Index<usize> for ReArr<T, N> {
    type Output = T;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        self.get(idx).unwrap()
    }
}

impl<T, const N: usize> std::ops::IndexMut<usize> for ReArr<T, N> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.get_mut(idx).unwrap()
    }
}

impl<T, const N: usize> IntoIterator for ReArr<T, N> {
    type Item = T;
    type IntoIter = Chain<Flatten<ArrayIter<Option<T>, N>>, VecIter<T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.arr.into_iter().flatten().chain(self.vec.into_iter())
    }
}

impl<T> std::iter::FromIterator<T> for ReArr<T, 0> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            arr: [],
            vec: iter.into_iter().collect(),
        }
    }
}

impl<T: Debug, const N: usize> Display for ReArr<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s.push('[');
        for (i, item) in self.iter().enumerate() {
            if i != 0 {
                s.push_str(", ");
            }
            write!(s, "{:?}", item)?;
        }
        s.push(']');
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::ReArr;

    const DEFAULT_TEST_COMBOVEC: ReArr<i32, 3> = rearr![1, 2, 3];

    #[test]
    fn make_new() {
        let mut cv = DEFAULT_TEST_COMBOVEC;
        cv.push(4);
        cv.push(5);
        println!("{}", cv);
        dbg!(&cv);
        assert_eq!(cv.get(0), Some(&1));
        assert_eq!(cv.get(1), Some(&2));
        assert_eq!(cv.get(2), Some(&3));
        assert_eq!(cv.get(3), Some(&4));
        assert_eq!(cv.last(), Some(&5));
        assert_eq!(cv.get(4), Some(&5));
        assert_eq!(cv.get(5), None);
        assert_eq!(cv.get_mut(0), Some(&mut 1));
    }

    #[test]
    fn iter() {
        let mut cv = DEFAULT_TEST_COMBOVEC;
        cv.push(4);
        assert_eq!(cv.iter().collect::<Vec<_>>(), vec![&1, &2, &3, &4]);
        assert_eq!(cv.into_iter().collect::<Vec<_>>(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn lengths() {
        let mut cv = DEFAULT_TEST_COMBOVEC;
        cv.push(4);
        assert_eq!(cv.len(), 4);
        assert_eq!(cv.stack_len(), 3);
        assert_eq!(cv.heap_len(), 1);
    }

    #[test]
    fn extend() {
        let mut cv = DEFAULT_TEST_COMBOVEC;
        cv.extend(vec![4, 5, 6]);
        cv.extend(DEFAULT_TEST_COMBOVEC);
        dbg!(&cv);
        assert_eq!(cv.len(), 9);
        assert_eq!(cv.stack_len(), 3);
        assert_eq!(cv.heap_len(), 6);
        assert_eq!(cv.to_vec(), vec![1, 2, 3, 4, 5, 6, 1, 2, 3]);
    }

    #[test]
    fn truncate_into_stack_push() {
        let mut cv = DEFAULT_TEST_COMBOVEC;
        cv.truncate(2);
        cv.push(3);
        assert_eq!(cv.len(), 3);
        assert_eq!(cv.stack_len(), 3);
        assert_eq!(cv.heap_len(), 0);
        assert_eq!(cv.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn truncate_into_stack() {
        let mut cv = DEFAULT_TEST_COMBOVEC;
        cv.truncate(2);
        assert_eq!(cv.len(), 2);
        assert_eq!(cv.stack_len(), 2);
        assert_eq!(cv.heap_len(), 0);
        assert_eq!(cv.to_vec(), vec![1, 2]);
    }

    #[test]
    fn truncate_into_heap() {
        let mut cv = DEFAULT_TEST_COMBOVEC;
        cv.extend(vec![4, 5, 6]);
        cv.truncate(4);
        assert_eq!(cv.len(), 4);
        assert_eq!(cv.stack_len(), 3);
        assert_eq!(cv.heap_len(), 1);
        assert_eq!(cv.to_vec(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn truncate_invalids() {
        let mut cv = DEFAULT_TEST_COMBOVEC;
        cv.truncate(4);
        cv.truncate(3);
        assert_eq!(cv.len(), 3);
        assert_eq!(cv.stack_len(), 3);
        assert_eq!(cv.heap_len(), 0);
        assert_eq!(cv.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn exarr_macro() {
        let item1 = rearr![1, 2, 3];
        println!("{}", item1);
        assert_eq!(item1.len(), 3);

        let item2 = rearr![5; 3];
        println!("{}", item2);
        assert_eq!(item2.len(), 3);

        let item3 = rearr![i32];
        println!("{}", item3);
        assert_eq!(item3.len(), 0);
        assert_eq!(item3.stack_capacity(), 0);

        let item4 = rearr![i32; 5];
        println!("{}", item4);
        assert_eq!(item4.len(), 0);
        assert_eq!(item4.stack_capacity(), 5);
    }
}
