#[cfg(feature = "alloc")]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use core::{
    array::IntoIter as ArrayIter,
    cmp::Ordering,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    iter::Flatten,
    ops,
};

/// Easy way to create a new [`ReArr`] with elements.
///
/// ## Examples
///
/// ```rust
/// use combo_vec::{re_arr, ReArr};
///
/// const SOME_ITEMS: ReArr<i8, 3> = re_arr![1, 2, 3];
/// const MANY_ITEMS: ReArr<u16, 90> = re_arr![5; 90];
/// const EXTRA_ITEMS: ReArr<&str, 5> = re_arr!["Hello", "world", "!"; None, None];
///
/// // Infer the type and size of the ReArr
/// const NO_STACK_F32: ReArr<f32, 0> = re_arr![];
///
/// // No const-initialization is needed to create a ComboVec with allocated elements on the stack
/// use std::collections::HashMap;
/// const EMPTY_HASHMAP_ALLOC: ReArr<HashMap<&str, i32>, 3> = re_arr![];
///
/// // Creating a new ReArr at compile time and doing this does have performance benefits
/// let my_re_arr = EMPTY_HASHMAP_ALLOC;
/// ```
#[macro_export]
macro_rules! re_arr {
    () => (
        $crate::ReArr::new()
    );
    ($elem:expr; $n:expr) => (
        $crate::ReArr::from_arr([Some($elem); $n])
    );
    ($($x:expr),+ $(,)?) => (
        $crate::ReArr::from_arr([$(Some($x)),+])
    );
    ($($x:expr),+; $($rest:expr),* $(,)?) => (
        $crate::ReArr::from_arr_and_len(&[$(Some($x)),+, $($rest),*])
    );
}

/// A [`ReArr`] is a fixed-size array with a variable number of elements.
///
/// Create a new [`ReArr`] using the [`re_arr!`] macro.
///
/// ## Examples
///
/// ```rust
/// use combo_vec::{re_arr, ReArr};
///
/// const SOME_ITEMS: ReArr<i8, 3> = re_arr![1, 2, 3];
/// const MANY_ITEMS: ReArr<u16, 90> = re_arr![5; 90];
///
/// // Infer the type and size of the ReArr
/// const NO_STACK_F32: ReArr<f32, 0> = re_arr![];
///
/// // No const-initialization is needed to create a ReArr with allocated elements on the stack
/// use std::collections::HashMap;
/// const EMPTY_HASHMAP_ALLOC: ReArr<HashMap<&str, i32>, 3> = re_arr![];
///
/// let mut my_re_arr = re_arr![1, 2, 3; None, None];
/// // Allocate an extra element on the heap
/// my_re_arr.push(4);
/// assert_eq!(my_re_arr.len(), 4);
/// // Truncate to only the first 2 elements
/// my_re_arr.truncate(2);
/// assert_eq!(my_re_arr.len(), 2);
/// // Fill the last element on the stack, then allocate the next two items on the heap
/// my_re_arr.extend([3, 4, 5]);
/// ```
pub struct ReArr<T, const N: usize> {
    pub(crate) arr: [Option<T>; N],
    arr_len: usize,
}

impl<T: Clone, const N: usize> Clone for ReArr<T, N> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            arr: self.arr.clone(),
            arr_len: self.arr_len,
        }
    }
}

impl<T: PartialOrd, const N: usize> PartialOrd for ReArr<T, N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T: Ord, const N: usize> Ord for ReArr<T, N> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
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

impl<T, const N: usize> Default for ReArr<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const N: usize> ReArr<T, N> {
    /// Create a new [`ReArr`] from an array.
    ///
    /// All slots must be populated with `Some` values until
    /// the first `None` value is encountered, or the end of the array is reached.
    /// After that, all remaining slots must be `None`.
    ///
    /// This function is forced to accept a reference to the array and then copy it
    /// due to <https://github.com/rust-lang/rust/issues/57349>
    ///
    /// This is used by the [`re_arr!`] macro.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let my_re_arr = ReArr::from_arr_and_len(&[Some(1), Some(2), Some(3), None, None]);
    /// let convenient_re_arr = re_arr![1, 2, 3; None, None];
    ///
    /// assert_eq!(my_re_arr, convenient_re_arr);
    /// assert_eq!(my_re_arr.len(), 3);
    /// assert_eq!(my_re_arr.capacity(), 5);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_arr_and_len(arr: &[Option<T>; N]) -> Self {
        let mut arr_len = 0;

        while arr_len < N {
            if arr[arr_len].is_none() {
                break;
            }

            arr_len += 1;
        }

        Self { arr: *arr, arr_len }
    }
}

impl<T, const N: usize> ReArr<T, N> {
    const DEFAULT_ARR_VALUE: Option<T> = None;

    /// Create a new, empty [`ReArr`] with the ability for `N` element to stored on the stack.
    ///
    /// This is used by the [`re_arr!`] macro, and you should consider using it instead.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// const RE_ARR: ReArr::<i32, 3> = re_arr![];
    /// let my_re_arr = ReArr::<i32, 3>::new();
    /// assert_eq!(my_re_arr, RE_ARR);
    /// ```
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
    ///
    /// This is used by the [`re_arr!`] macro.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let my_re_arr = ReArr::from_arr([Some(1), Some(2), Some(3)]);
    /// let convenient_re_arr = re_arr![1, 2, 3];
    ///
    /// assert_eq!(my_re_arr, convenient_re_arr);
    /// assert_eq!(my_re_arr.len(), 3);
    /// assert_eq!(my_re_arr.capacity(), 3);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_arr(arr: [Option<T>; N]) -> Self {
        Self { arr, arr_len: N }
    }

    // Create a new [`ReArr`] from an iterator reference, taking up to N items
    // 
    // Allows for initialization without consuming the iterator, leaving its
    // remaining content for another procedure.
    // 
    // This is useful for ComboVec::from_iter, which needs to initialise both
    // a ReArr and a Vec.
    pub(crate) fn from_iter_ref(iter: &mut impl Iterator<Item = T>) -> Self {
        let mut re_arr = Self::new();
        for _ in 0..N {
            if let Some(val) = iter.next() {
                re_arr.push(val);
                continue;
            }
            break;
        }
        re_arr
    }

    /// Push an element to the end of the array.
    ///
    /// ## Panics
    ///
    /// Panics if the array is full.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None];
    /// my_re_arr.push(4);
    ///
    /// assert_eq!(my_re_arr.len(), 4);
    /// assert_eq!(my_re_arr.capacity(), 4);
    /// assert_eq!(my_re_arr.to_vec(), vec![1, 2, 3, 4]);
    /// ```
    #[inline]
    pub fn push(&mut self, val: T) {
        self.arr[self.arr_len] = Some(val);
        self.arr_len += 1;
    }

    /// Remove the last element from the array and return it, or None if it is empty.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.pop(), Some(3));
    /// assert_eq!(my_re_arr.pop(), Some(2));
    /// assert_eq!(my_re_arr.pop(), Some(1));
    /// assert_eq!(my_re_arr.pop(), None);
    /// ```
    #[inline]
    pub const fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.arr_len -= 1;
            self.arr[self.arr_len].take()
        }
    }

    /// Get any element from the array as a reference, returning `None` if out of bounds.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.get(0), Some(&1));
    /// assert_eq!(my_re_arr.get(1), Some(&2));
    /// assert_eq!(my_re_arr.get(2), Some(&3));
    /// assert_eq!(my_re_arr[2], 3);
    /// assert_eq!(my_re_arr.get(3), None);
    /// assert_eq!(my_re_arr.get(4), None);
    /// assert_eq!(my_re_arr.get(5), None);
    /// ```
    #[must_use]
    #[inline]
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.arr.get(idx).and_then(|item| item.as_ref())
    }

    /// Get any element from the array as a mutable reference, `None` if out of bounds.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.get_mut(0), Some(&mut 1));
    /// assert_eq!(my_re_arr.get_mut(1), Some(&mut 2));
    /// my_re_arr[1] = 4;
    /// assert_eq!(my_re_arr.get_mut(1), Some(&mut 4));
    /// assert_eq!(my_re_arr.get_mut(2), Some(&mut 3));
    /// assert_eq!(my_re_arr.get_mut(3), None);
    /// assert_eq!(my_re_arr.get_mut(4), None);
    /// assert_eq!(my_re_arr.get_mut(5), None);
    /// ```
    #[must_use]
    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.arr.get_mut(idx).and_then(|item| item.as_mut())
    }

    /// How many elements are currently stored.
    ///
    /// This is not the same as the capacity of the internal array.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.len(), 3);
    /// my_re_arr.push(4);
    /// assert_eq!(my_re_arr.len(), 4);
    /// my_re_arr.pop();
    /// assert_eq!(my_re_arr.len(), 3);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.arr_len
    }

    /// How many elements can be stored.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.capacity(), 4);
    /// my_re_arr.push(4);
    /// assert_eq!(my_re_arr.capacity(), 4);
    /// my_re_arr.pop();
    /// assert_eq!(my_re_arr.capacity(), 4);
    /// ```
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Reduce the number of elements to the given length.
    ///
    /// If `new_len` is greater than the current length, this has no effect.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.len(), 3);
    /// my_re_arr.truncate(2);
    /// assert_eq!(my_re_arr.len(), 2);
    /// ```
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.arr[len..].iter_mut().for_each(|x| *x = None);
        self.arr_len = self.arr_len.min(len);
    }

    /// Remove all elements from the array.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.len(), 3);
    /// my_re_arr.clear();
    /// assert_eq!(my_re_arr.len(), 0);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.arr.iter_mut().for_each(|x| *x = None);
        self.arr_len = 0;
    }

    /// Get the first element, returning `None` if there are no elements.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.first(), Some(&1));
    /// ```
    #[inline]
    pub const fn first(&self) -> Option<&T> {
        if N == 0 {
            None
        } else {
            self.arr[0].as_ref()
        }
    }

    /// Get the first element as a mutable reference, returning `None` if there are no elements.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.first_mut(), Some(&mut 1));
    /// ```
    #[inline]
    pub const fn first_mut(&mut self) -> Option<&mut T> {
        if N == 0 {
            None
        } else {
            self.arr[0].as_mut()
        }
    }

    /// Get the last element, returning `None` if there are no elements.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.last(), Some(&3));
    /// ```
    #[inline]
    pub const fn last(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.arr[self.arr_len - 1].as_ref()
        }
    }

    /// Get the last element as a mutable reference, returning `None` if there are no elements.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.last_mut(), Some(&mut 3));
    /// ```
    #[inline]
    pub const fn last_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            self.arr[self.arr_len - 1].as_mut()
        }
    }

    /// Check if there are no elements.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let my_re_arr = re_arr![1, 2, 3; None];
    /// assert_eq!(my_re_arr.is_empty(), false);
    ///
    /// let empty_re_arr = ReArr::<i32, 3>::new();
    /// assert_eq!(empty_re_arr.is_empty(), true);
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.arr_len == 0
    }

    /// Get an iterator over the elements of the array.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.iter().collect::<Vec<_>>(), vec![&1, &2, &3]);
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.arr.iter().flatten()
    }

    /// Get an iterator over the elements of the array, returning mutable references.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.iter_mut().collect::<Vec<_>>(), vec![&mut 1, &mut 2, &mut 3]);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.arr.iter_mut().flatten()
    }

    /// Extend this array with all the elements from the given iterator.
    ///
    /// ## Panics
    ///
    /// Panics if the iterator tries to push more elements than the internal array can hold.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None, None, None];
    /// assert_eq!(my_re_arr.len(), 3);
    /// assert_eq!(my_re_arr.capacity(), 6);
    ///
    /// my_re_arr.extend([4, 5, 6]);
    /// assert_eq!(my_re_arr.len(), 6);
    /// ```
    #[inline]
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|x| self.push(x));
    }

    /// Get this [`ReArr`] transformed into a [`Vec`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.into_vec(), vec![1, 2, 3]);
    /// ```
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.into_iter().collect()
    }

    /// Get this [`ReArr`] represented as a [`Vec`], borrowing data instead of moving it.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.ref_vec(), vec![&1, &2, &3]);
    /// ```
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn ref_vec(&self) -> Vec<&T> {
        self.iter().collect()
    }
}

impl<T: Clone, const N: usize> ReArr<T, N> {
    /// Get this [`ReArr`] represented as a [`Vec`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.to_vec(), vec![1, 2, 3]);
    /// // my_re_arr is still usable
    /// assert_eq!(my_re_arr.len(), 3);
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
    /// ## Panics
    ///
    /// If `new_len` is greater than the length of the internal array.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None, None];
    ///
    /// assert_eq!(my_re_arr.len(), 3);
    /// my_re_arr.resize(5, 4);
    /// assert_eq!(my_re_arr.len(), 5);
    /// assert_eq!(my_re_arr.to_vec(), vec![1, 2, 3, 4, 4]);
    /// my_re_arr.resize(2, 4);
    /// assert_eq!(my_re_arr.len(), 2);
    /// assert_eq!(my_re_arr.to_vec(), vec![1, 2]);
    /// ```
    pub fn resize(&mut self, new_len: usize, val: T) {
        assert!(new_len <= N, "new length cannot be greater than the internal array length");

        if new_len > self.arr_len {
            self.arr[self.arr_len..new_len].fill(Some(val));
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
    /// If `new_len` is less than `len`, the [`ReArr`] is truncated.
    ///
    /// ## Panics
    ///
    /// If `new_len` is greater than the length of the internal array.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None, None];
    ///
    /// assert_eq!(my_re_arr.len(), 3);
    /// my_re_arr.resize_with(5, Default::default);
    /// assert_eq!(my_re_arr.len(), 5);
    /// assert_eq!(my_re_arr.to_vec(), vec![1, 2, 3, 0, 0]);
    /// my_re_arr.resize_with(2, Default::default);
    /// assert_eq!(my_re_arr.len(), 2);
    /// assert_eq!(my_re_arr.to_vec(), vec![1, 2]);
    /// ```
    pub fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, mut f: F) {
        assert!(new_len <= N, "new length cannot be greater than the internal array length");

        if new_len > self.arr_len {
            self.arr[self.arr_len..new_len].fill(Some(f()));
        } else {
            self.arr[new_len..].fill(None);
        }

        self.arr_len = new_len;
    }

    /// Removes and returns the element at position with a valid index, shifting all elements after it to the left.
    ///
    /// ## Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.remove(1), 2);
    /// assert_eq!(my_re_arr.len(), 2);
    /// assert_eq!(my_re_arr.to_vec(), vec![1, 3]);
    /// ```
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
    /// ## Panics
    ///
    /// Panics if `index` is out of bounds, or if it is the last value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{re_arr, ReArr};
    ///
    /// let mut my_re_arr = re_arr![1, 2, 3; None];
    ///
    /// assert_eq!(my_re_arr.swap_remove(0), 1);
    /// assert_eq!(my_re_arr.len(), 2);
    /// assert_eq!(my_re_arr.to_vec(), vec![3, 2]);
    /// ```
    #[inline]
    pub const fn swap_remove(&mut self, index: usize) -> T {
        let last_value = self.pop().unwrap();
        self.arr[index].replace(last_value).unwrap()
    }
}

#[cfg(feature = "alloc")]
impl<T: ToString, const N: usize> ReArr<T, N> {
    /// Joins the [`ReArr`] into a string with a separator.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::re_arr;
    ///
    /// let x = re_arr![1, 2, 3];
    /// assert_eq!(x.join(", "), "1, 2, 3");
    /// ```
    pub fn join(&self, sep: &str) -> String {
        self.iter()
            .enumerate()
            .fold(String::with_capacity(self.arr_len), |mut s, (i, item)| {
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

impl<T, const N: usize> FromIterator<T> for ReArr<T, N> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        ReArr::from_iter_ref(&mut iter.into_iter())
    }
}

impl<T: Debug, const N: usize> Debug for ReArr<T, N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ReArr")
            .field("arr", &self.arr)
            .field("arr_len", &self.arr_len)
            .finish()
    }
}

impl<T: Debug, const N: usize> Display for ReArr<T, N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_list().entries(self.arr.iter().flatten()).finish()
    }
}
