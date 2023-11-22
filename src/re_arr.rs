#[cfg(feature = "alloc")]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use core::{
    array::IntoIter as ArrayIter,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::Hash,
    iter::Flatten,
    ops,
};

/// Easy way to create a new [`ReArr`] with elements.
#[macro_export]
macro_rules! re_arr {
    () => (
        $crate::ReArr::new()
    );
    ($type:ty) => (
        $crate::ReArr::<$type, 16>::new()
    );
    ($type:ty; $n:literal) => (
        $crate::ReArr::<$type, $n>::new()
    );
    ($elem:expr; $n:expr) => (
        $crate::ReArr::from_arr([Some($elem); $n])
    );
    ($($x:expr),+ $(,)?) => (
        $crate::ReArr::from_arr([$(Some($x)),+])
    );
}

/// A [`ReArr`] is a fixed-size array with a variable number of elements.
#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct ReArr<T, const N: usize> {
    arr: [Option<T>; N],
    arr_len: usize,
}

impl<T, const N: usize> Default for ReArr<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> ReArr<T, N> {
    const DEFAULT_ARR_VALUE: Option<T> = None;

    /// Create a new [`ReArr`].
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            arr: [Self::DEFAULT_ARR_VALUE; N],
            arr_len: 0,
        }
    }

    /// Create a new [`ReArr`] from an array.
    ///
    /// All slots must be populated with `Some` values.
    #[must_use]
    #[inline]
    pub const fn from_arr(arr: [Option<T>; N]) -> Self {
        Self { arr, arr_len: N }
    }

    /// Create a new [`ReArr`] from an array.
    ///
    /// All slots must be populated with `Some` values until
    /// the first `None` value is encountered, or the end of the array is reached.
    /// After that, all remaining slots must be `None`.
    #[must_use]
    #[inline]
    pub const fn from_arr_and_len(arr: [Option<T>; N], arr_len: usize) -> Self {
        Self { arr, arr_len }
    }

    /// Push an element to the end of the array.
    #[inline]
    pub fn push(&mut self, val: T) {
        self.arr[self.arr_len] = Some(val);
        self.arr_len += 1;
    }

    /// Remove the last element from the array and return it, or None if it is empty.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.arr_len > 0 {
            self.arr_len -= 1;
            self.arr[self.arr_len].take()
        } else {
            None
        }
    }

    /// Get any element from the array as a reference, returning `None` if out of bounds.
    #[must_use]
    #[inline]
    pub const fn get(&self, idx: usize) -> Option<&T> {
        self.arr[idx].as_ref()
    }

    /// Get any element from the array as a mutable reference, `None` if out of bounds.
    #[must_use]
    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.arr[idx].as_mut()
    }

    /// How many elements are currently stored.
    #[inline]
    pub const fn len(&self) -> usize {
        self.arr_len
    }

    /// How many elements can be stored.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Reduce the number of elements to the given length.
    ///
    /// If `new_len` is greater than the current length, this has no effect.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.arr[len..].iter_mut().for_each(|x| *x = None);
        self.arr_len = self.arr_len.min(len);
    }

    /// Remove all elements from the array.
    #[inline]
    pub fn clear(&mut self) {
        self.arr.iter_mut().for_each(|x| *x = None);
        self.arr_len = 0;
    }

    /// Get the first element, returning `None` if there are no elements.
    #[inline]
    pub const fn first(&self) -> Option<&T> {
        if N == 0 {
            None
        } else {
            self.arr[0].as_ref()
        }
    }

    /// Get the first element as a mutable reference, returning `None` if there are no elements.
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut T> {
        if N == 0 {
            None
        } else {
            self.arr[0].as_mut()
        }
    }

    /// Get the last element, returning `None` if there are no elements.
    #[inline]
    pub const fn last(&self) -> Option<&T> {
        if N == 0 {
            None
        } else {
            self.arr[N - 1].as_ref()
        }
    }

    /// Get the last element as a mutable reference, returning `None` if there are no elements.
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if N == 0 {
            None
        } else {
            self.arr[N - 1].as_mut()
        }
    }

    /// Check if there are no elements.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an iterator over the elements of the array.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.arr[..self.arr_len].iter().filter_map(Option::as_ref)
    }

    /// Get an iterator over the elements of the array, returning mutable references.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.arr[..self.arr_len].iter_mut().filter_map(Option::as_mut)
    }

    /// Extend this array with all the elements from the given iterator.
    #[inline]
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|x| self.push(x));
    }

    /// Get this [`ReArr`] transformed into a [`Vec`].
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.into_iter().collect()
    }

    /// Get this [`ReArr`] represented as a [`Vec`], borrowing data instead of cloning it.
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn ref_vec(&self) -> Vec<&T> {
        self.iter().collect()
    }
}

impl<T: Clone, const N: usize> ReArr<T, N> {
    /// Get this [`ReArr`] represented as a [`Vec`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let x = combo_vec![1, 2, 3];
    /// assert_eq!(x.to_vec(), vec![1, 2, 3]);
    /// ```
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

    /// Resizes the [`ReArr`] in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the [`ReArr`] is extended by the
    /// difference, with each additional slot filled with `val`.
    ///
    /// If `new_len` is less than `len`, the [`ReArr`] is truncated.
    ///
    /// # Panics
    ///
    /// If `new_len` is greater than the length of the internal array.
    pub fn resize(&mut self, new_len: usize, val: T) {
        assert!(new_len <= N, "new length cannot be greater than the internal array length");

        if new_len > self.len() {
            self.arr[..new_len].fill(Some(val));
        } else {
            self.arr[new_len..].fill(None);
        }

        self.arr_len = new_len;
    }

    /// Resizes the [`ReArr`] in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the [`ReArr`] is extended
    /// with the result of calling the closure `f`.
    ///
    /// If `new_len` is less than `len`, the [`ReArr`] is truncated.vec()
    ///
    /// # Panics
    ///
    /// If `new_len` is greater than the length of the internal array.
    /// ```
    pub fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, mut f: F) {
        assert!(new_len <= N, "new length cannot be greater than the internal array length");

        if new_len > self.len() {
            self.arr[..new_len].fill(Some(f()));
        } else {
            self.arr[new_len..].fill(None);
        }

        self.arr_len = new_len;
    }

    /// Removes and returns the element at position with a valid index, shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        let val = self.arr[index].take().unwrap();

        for i in index..self.arr_len - 1 {
            self.arr[i] = self.arr[i + 1].take();
        }

        self.arr_len -= 1;

        val
    }

    /// Removes an element from the `ReArr` and returns it.
    ///
    /// The removed element is replaced by the last element of the `ReArr`.
    ///
    /// This does not preserve ordering, but is O(1). If you need to preserve the element order, use remove instead.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds, or if it is the last value.
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        let last_value = self.pop().unwrap();
        self.arr[index].replace(last_value).unwrap()
    }
}

#[cfg(feature = "alloc")]
impl<T: ToString, const N: usize> ReArr<T, N> {
    /// Joins the [`ReArr`] into a string with a separator.
    pub fn join(&self, sep: &str) -> String {
        self.iter()
            .enumerate()
            .fold(String::with_capacity(self.len()), |mut s, (i, item)| {
                if i != 0 {
                    s.push_str(sep);
                }

                s.push_str(&item.to_string());
                s
            })
    }
}

impl<T, const N: usize> ops::Index<usize> for ReArr<T, N> {
    type Output = T;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        self.arr[idx].as_ref().unwrap()
    }
}

impl<T, const N: usize> ops::IndexMut<usize> for ReArr<T, N> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.arr[idx].as_mut().unwrap()
    }
}

impl<T, const N: usize> IntoIterator for ReArr<T, N> {
    type Item = T;
    type IntoIter = Flatten<ArrayIter<Option<T>, N>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.arr.into_iter().flatten()
    }
}

impl<T: Debug, const N: usize> Display for ReArr<T, N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_list().entries(self.arr.iter().flatten()).finish()
    }
}
