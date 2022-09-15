#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! An array that can be resized at runtime, without moving any data off the stack.
//! 
//! This works by allocating an array of `T` on the stack, and then using a Vec on the heap for overflow.
//! 
//! The stack-allocated array is always used to store the first `N` elements, even when the array is resized.
//! 
//! ## Usage
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
//! assert_eq!(resizeable_vec.len(), 5);
//! assert_eq!(resizeable_vec[1], 2);
//! assert_eq!(resizeable_vec.last(), Some(&5));
//! assert_eq!(resizeable_vec.to_vec(), vec![1, 2, 3, 4, 5]);
//! assert_eq!(resizeable_vec.stack_len(), 3);
//! assert_eq!(resizeable_vec.heap_len(), 2);
//! ```

use std::{
    array::IntoIter as ArrayIter,
    fmt::{Debug, Display, Write},
    iter::{Chain, Flatten},
    vec::IntoIter as VecIter,
};

#[macro_export]
macro_rules! rearr {
    ($type:ty) => (
        ReArr::<$type, 0>::new_empty()
    );
    ($type:ty; $n:literal) => (
        ReArr::<$type, $n>::with_capacity()
    );
    ($elem:expr; $n:expr) => (
        ReArr::from_arr([Some($elem); $n])
    );
    ($($x:expr),+ $(,)?) => (
        ReArr::from_arr([$(Some($x)),+])
    );
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReArr<T, const N: usize> {
    arr: [Option<T>; N],
    vec: Vec<T>,
}

impl<T> ReArr<T, 0> {
    #[inline]
    #[must_use]
    pub const fn new_empty() -> Self {
        Self { arr: [], vec: Vec::new() }
    }
}

impl<T: Copy, const N: usize> ReArr<T, N> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self { arr: [None; N], vec: Vec::new() }
    }
}

impl<T: Copy + Default, const N: usize> Default for ReArr<T, N> {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> ReArr<T, N> {
    const DEFAULT_ARR_VALUE: Option<T> = None;

    #[must_use]
    #[inline]
    pub const fn with_capacity() -> Self {
        Self {
            arr: [Self::DEFAULT_ARR_VALUE; N],
            vec: Vec::new(),
        }
    }

    #[must_use]
    #[inline]
    pub const fn from_arr(arr: [Option<T>; N]) -> Self {
        Self { arr, vec: Vec::new() }
    }

    #[inline]
    pub fn push(&mut self, val: T) {
        let stack_len = self.stack_len();
        if stack_len < N {
            self.arr[stack_len] = Some(val);
        } else {
            self.vec.push(val);
        }
    }

    #[must_use]
    #[inline]
    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < N {
            self.arr[idx].as_ref()
        } else {
            self.vec.get(idx - N)
        }
    }

    #[must_use]
    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < N {
            self.arr[idx].as_mut()
        } else {
            self.vec.get_mut(idx - N)
        }
    }

    #[inline]
    pub fn stack_len(&self) -> usize {
        self.arr.iter().flatten().count()
    }

    #[inline]
    pub fn heap_len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.stack_len() + self.heap_len()
    }

    #[inline]
    pub const fn stack_capacity(&self) -> usize {
        N
    }

    #[inline]
    pub fn heap_capacity(&self) -> usize {
        self.vec.capacity()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.stack_capacity() + self.heap_capacity()
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len > self.len() {
            // do nothing
        } else if len >= N {
            self.vec.truncate(len - N);
        } else {
            self.arr[len..].iter_mut().for_each(|x| *x = None);
            self.vec = Vec::new();
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.arr.iter_mut().for_each(|x| *x = None);
        self.vec.clear();
    }

    #[inline]
    pub fn last(&self) -> Option<&T> {
        self.vec.last().or_else(|| self.arr[N - 1].as_ref())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty() && self.arr.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.arr.iter().filter_map(Option::as_ref).chain(self.vec.iter())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.arr.iter_mut().filter_map(Option::as_mut).chain(self.vec.iter_mut())
    }

    #[inline]
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for e in iter {
            self.push(e);
        }
    }
}

impl<T: Clone, const N: usize> ReArr<T, N> {
    #[inline]
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

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
        let mut vec = Self::new_empty();
        for item in iter {
            vec.push(item);
        }
        vec
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

    const DEFAULT_TEST_COMBOVEC: ReArr<i32, 3> = ReArr::from_arr([Some(1), Some(2), Some(3)]);

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
