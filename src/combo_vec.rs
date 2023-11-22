use alloc::{
    string::{String, ToString},
    vec::{IntoIter as VecIter, Vec},
};
use core::{
    array::IntoIter as ArrayIter,
    cmp::Ordering,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    iter::{Chain, Flatten},
    ops,
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
///
/// // Infer the type and size of the ReArr
/// const NO_STACK_F32: ReArr<f32, 0> = rearr![];
///
/// // No const-initialization is needed to create a ReArr with allocated elements on the stack
/// use std::collections::HashMap;
/// const EMPTY_HASHMAP_ALLOC: ReArr<HashMap<&str, i32>, 3> = rearr![];
///
/// let my_rearr = rearr![1, 2, 3];
/// ```
#[macro_export]
macro_rules! rearr {
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
///
/// // Infer the type and size of the ReArr
/// const NO_STACK_F32: ReArr<f32, 0> = rearr![];
///
/// // No const-initialization is needed to create a ReArr with allocated elements on the stack
/// use std::collections::HashMap;
/// const EMPTY_HASHMAP_ALLOC: ReArr<HashMap<&str, i32>, 3> = rearr![];
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
pub struct ReArr<T, const N: usize> {
    arr: [Option<T>; N],
    arr_len: usize,
    vec: Vec<T>,
}

impl<T: Clone, const N: usize> Clone for ReArr<T, N> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            arr: self.arr.clone(),
            arr_len: self.arr_len,
            vec: self.vec.clone(),
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

impl<T: Default, const N: usize> Default for ReArr<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> ReArr<T, N> {
    const DEFAULT_ARR_VALUE: Option<T> = None;

    /// Create a new, empty [`ReArr`] with with the ability for `N` element to be allocated on the stack.
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
            arr_len: 0,
            vec: Vec::new(),
        }
    }

    /// Allocate more memory to what can be stored on the heap.
    ///
    /// Note that this function is not required to add more items, but can be used as an optimization to avoid excessive reallocations when adding many items.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![1, 2, 3];
    /// let my_rearr2 = rearr![4, 5, 6];
    ///
    /// // How much space can be stored in my_rearr currently (length of stack + number of items in heap)
    /// // This is needed because if the stack isn't full and can store all our items, we don't want to allocate more space on the heap.
    /// let my_rearr_capacity = my_rearr2.stack_capacity() + my_rearr.heap_len();
    /// let extra_capacity = my_rearr_capacity - my_rearr.len();
    ///
    /// // Check if we need to reallocate
    /// if extra_capacity < my_rearr2.len() {
    ///     my_rearr.reserve(my_rearr2.len() - extra_capacity);
    /// }
    ///
    /// assert!(my_rearr.capacity() >= my_rearr.len() + my_rearr2.len());
    ///
    /// // Extend my_rearr with my_rearr2
    /// my_rearr.extend(my_rearr2);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
    }

    /// Create a resizable array from a fixed size array.
    ///
    /// Use Some is used for initialized values, and None is used for uninitialized values.
    ///
    /// This is used by the [`rearr!`] macro, and you should consider using it instead.
    /// Note that the macro can't create mixed initialized and uninitialized arrays, only one or the other.
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
        Self {
            arr,
            arr_len: N,
            vec: Vec::new(),
        }
    }

    /// Push an element to the end of the array.
    ///
    /// If the array is full, the element will be pushed to the heap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![1, 2, 3];
    /// my_rearr.push(4);
    /// assert_eq!(my_rearr.to_vec(), vec![1, 2, 3, 4]);
    /// ```
    #[inline]
    pub fn push(&mut self, val: T) {
        if self.arr_len < N {
            self.arr[self.arr_len] = Some(val);
            self.arr_len += 1;
        } else {
            self.vec.push(val);
        }
    }

    /// Remove the last element from the array and return it, or None if it is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![1, 2, 3];
    /// assert_eq!(my_rearr.pop(), Some(3));
    /// assert_eq!(my_rearr.pop(), Some(2));
    /// my_rearr.extend(rearr![4, 5, 6]);
    /// assert_eq!(my_rearr.pop(), Some(6));
    /// assert_eq!(my_rearr.pop(), Some(5));
    /// assert_eq!(my_rearr.pop(), Some(4));
    /// assert_eq!(my_rearr.pop(), Some(1));
    /// assert_eq!(my_rearr.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if !self.vec.is_empty() {
            self.vec.pop()
        } else if self.arr_len > 0 {
            self.arr_len -= 1;
            self.arr[self.arr_len].take()
        } else {
            None
        }
    }

    /// Get any element from the array as a reference, returning `None` if out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let my_rearr = rearr![1, 2, 3];
    /// assert_eq!(my_rearr.get(0), Some(&1));
    /// assert_eq!(my_rearr.get(1), Some(&2));
    /// assert_eq!(my_rearr.get(2), Some(&3));
    /// assert_eq!(my_rearr.get(3), None);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![1, 2, 3];
    ///
    /// if let Some(x) = my_rearr.get_mut(0) {
    ///     *x = 4;
    /// }
    ///
    /// assert_eq!(my_rearr.to_vec(), vec![4, 2, 3]);
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![1, 2, 3];
    /// assert_eq!(my_rearr.spilled(), false);
    ///
    /// my_rearr.push(4);
    /// assert_eq!(my_rearr.spilled(), true);
    /// ```
    #[inline]
    pub fn spilled(&self) -> bool {
        self.heap_len() > 0
    }

    /// How many elements are currently stored on the stack.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![1, 2, 3];
    /// assert_eq!(my_rearr.stack_len(), 3);
    /// my_rearr.push(4);
    /// assert_eq!(my_rearr.stack_len(), 3);
    /// ```
    #[inline]
    pub const fn stack_len(&self) -> usize {
        self.arr_len
    }

    /// How many elements are currently stored on the heap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![1, 2, 3];
    /// assert_eq!(my_rearr.heap_len(), 0);
    /// my_rearr.push(4);
    /// assert_eq!(my_rearr.heap_len(), 1);
    /// ```
    #[inline]
    pub fn heap_len(&self) -> usize {
        self.vec.len()
    }

    /// How many elements are currently stored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![1, 2, 3];
    /// assert_eq!(my_rearr.len(), 3);
    /// my_rearr.push(4);
    /// assert_eq!(my_rearr.len(), 4);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.stack_len() + self.heap_len()
    }

    /// How many elements can be stored on the stack.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![i32; 3];
    /// assert_eq!(my_rearr.len(), 0);
    /// assert_eq!(my_rearr.stack_capacity(), 3);
    /// ```
    #[inline]
    pub const fn stack_capacity(&self) -> usize {
        N
    }

    /// How many elements can be stored on the currently allocated heap.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![i32; 3];
    /// assert_eq!(my_rearr.len(), 0);
    /// assert_eq!(my_rearr.heap_capacity(), 0);
    /// ```
    #[inline]
    pub fn heap_capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// How many elements can be stored without reallocating anything.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![i32; 3];
    /// assert_eq!(my_rearr.len(), 0);
    /// assert_eq!(my_rearr.capacity(), 3);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.stack_capacity() + self.heap_capacity()
    }

    /// Reduce the number of elements to the given length.
    ///
    /// If `new_len` is greater than the current length, this has no effect.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![1, 2, 3, 4, 5];
    /// my_rearr.truncate(3);
    /// assert_eq!(my_rearr.to_vec(), vec![1, 2, 3]);
    /// ```
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len > self.len() {
            // do nothing
        } else if len >= N {
            self.vec.truncate(len - N);
        } else {
            self.arr[len..].iter_mut().for_each(|x| *x = None);
            self.arr_len = len;
            self.vec.clear();
        }
    }

    /// Remove all elements from the array.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![1, 2, 3, 4, 5];
    /// my_rearr.clear();
    /// assert!(my_rearr.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.arr.iter_mut().for_each(|x| *x = None);
        self.arr_len = 0;
        self.vec.clear();
    }

    /// Get the first element, returning `None` if there are no elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let my_rearr = rearr![1, 2, 3];
    /// assert_eq!(my_rearr.first(), Some(&1));
    /// ```
    #[inline]
    pub fn first(&self) -> Option<&T> {
        if N == 0 {
            self.vec.first()
        } else {
            self.arr[0].as_ref().or_else(|| self.vec.first())
        }
    }

    /// Get the first element as a mutable reference, returning `None` if there are no elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![1, 2, 3];
    /// *my_rearr.first_mut().unwrap() = 4;
    /// assert_eq!(my_rearr.first(), Some(&4));
    /// ```
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut T> {
        if N == 0 {
            self.vec.first_mut()
        } else {
            self.arr[0].as_mut().or_else(|| self.vec.first_mut())
        }
    }

    /// Get the last element, returning `None` if there are no elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let my_rearr = rearr![1, 2, 3];
    /// assert_eq!(my_rearr.last(), Some(&3));
    /// ```
    #[inline]
    pub fn last(&self) -> Option<&T> {
        if N == 0 {
            self.vec.last()
        } else {
            self.vec.last().or_else(|| self.arr[N - 1].as_ref())
        }
    }

    /// Get the last element as a mutable reference, returning `None` if there are no elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![1, 2, 3];
    /// *my_rearr.last_mut().unwrap() = 4;
    /// assert_eq!(my_rearr.last(), Some(&4));
    /// ```
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if N == 0 {
            self.vec.last_mut()
        } else {
            self.vec.last_mut().or_else(|| self.arr[N - 1].as_mut())
        }
    }

    /// Check if there are no elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut my_rearr = rearr![i32; 3];
    /// assert!(my_rearr.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an iterator over the elements of the array.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let x = rearr![1, 2, 3];
    /// let mut iter = x.iter();
    ///
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), Some(&3));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.arr[..self.arr_len]
            .iter()
            .filter_map(Option::as_ref)
            .chain(self.vec.iter())
    }

    /// Get an iterator over the elements of the array, returning mutable references.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut x = rearr![1, 2, 3];
    /// for i in x.iter_mut() {
    ///    *i += 1;
    /// }
    /// assert_eq!(x.to_vec(), vec![2, 3, 4]);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.arr[..self.arr_len]
            .iter_mut()
            .filter_map(Option::as_mut)
            .chain(self.vec.iter_mut())
    }

    /// Extend this array with all the elements from the given iterator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut x = rearr![1, 2, 3];
    /// x.extend(vec![4, 5, 6]);
    /// assert_eq!(x.to_vec(), vec![1, 2, 3, 4, 5, 6]);
    /// ```
    #[inline]
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|x| self.push(x));
    }

    /// Get this [`ReArr`] transformed into a [`Vec`].
    ///
    /// [`Vec`]: std::vec::Vec
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let x = rearr![1, 2, 3];
    /// assert_eq!(x.into_vec(), vec![1, 2, 3]);
    /// ```
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.into_iter().collect()
    }

    /// Get this [`ReArr`] represented as a [`Vec`], borrowing data instead of cloning it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let x = rearr![1, 2, 3];
    /// assert_eq!(x.ref_vec(), vec![&1, &2, &3]);
    /// ```
    #[inline]
    pub fn ref_vec(&self) -> Vec<&T> {
        self.iter().collect()
    }
}

impl<T: Clone, const N: usize> ReArr<T, N> {
    /// Get this [`ReArr`] represented as a [`Vec`].
    ///
    /// [`Vec`]: std::vec::Vec
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let x = rearr![1, 2, 3];
    /// assert_eq!(x.to_vec(), vec![1, 2, 3]);
    /// ```
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
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut x = rearr![1, 2, 3];
    /// x.resize(5, 4);
    /// assert_eq!(x.to_vec(), vec![1, 2, 3, 4, 4]);
    /// x.resize(2, 5);
    /// assert_eq!(x.to_vec(), vec![1, 2]);
    /// x.resize(5, 6);
    /// assert_eq!(x.to_vec(), vec![1, 2, 6, 6, 6]);
    /// ```
    pub fn resize(&mut self, new_len: usize, val: T) {
        if new_len >= N {
            let num_items = self.len();
            if num_items < N {
                self.arr[num_items..N].fill(Some(val.clone()));
                self.arr_len = N;
            }

            self.vec.resize(new_len - N, val);
            return;
        } else if new_len > self.len() {
            self.arr[..new_len].fill(Some(val));
        } else {
            self.arr[new_len..].fill(None);
        }

        self.arr_len = new_len;
        self.vec.clear();
    }

    /// Resizes the [`ReArr`] in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the [`ReArr`] is extended by the
    /// difference, with each additional slot filled with the result of calling
    /// the closure `f`.
    ///
    /// If `new_len` is less than `len`, the [`ReArr`] is truncated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut x = rearr![1, 2, 3];
    /// x.resize_with(5, Default::default);
    /// assert_eq!(x.to_vec(), vec![1, 2, 3, 0, 0]);
    /// x.resize_with(2, Default::default);
    /// assert_eq!(x.to_vec(), vec![1, 2]);
    /// x.resize_with(5, Default::default);
    /// assert_eq!(x.to_vec(), vec![1, 2, 0, 0, 0]);
    /// ```
    pub fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, mut f: F) {
        if new_len >= N {
            let num_items = self.len();
            if num_items < N {
                self.arr[num_items..N].fill(Some(f()));
                self.arr_len = N;
            }

            self.vec.resize_with(new_len - N, f);
            return;
        } else if new_len > self.len() {
            self.arr[..new_len].fill(Some(f()));
        } else {
            self.arr[new_len..].fill(None);
        }

        self.arr_len = new_len;
        self.vec.clear();
    }

    /// Removes and returns the element at position with a valid index, shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut x = rearr![1, 2, 3];
    /// assert_eq!(x.remove(1), 2);
    /// assert_eq!(x.to_vec(), vec![1, 3]);
    /// x.extend([4, 5, 6]);
    /// assert_eq!(x.to_vec(), vec![1, 3, 4, 5, 6]);
    /// assert_eq!(x.remove(3), 5);
    /// assert_eq!(x.to_vec(), vec![1, 3, 4, 6]);
    /// ```
    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        if index >= N {
            self.vec.remove(index - N)
        } else {
            let val = self.arr[index].take().unwrap();

            for i in index..self.arr_len - 1 {
                self.arr[i] = self.arr[i + 1].take();
            }

            if self.vec.is_empty() {
                self.arr_len -= 1;
            } else {
                self.arr[N - 1] = Some(self.vec.remove(0));
            }

            val
        }
    }

    /// Removes an element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    ///
    /// This does not preserve ordering, but is O(1). If you need to preserve the element order, use remove instead.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds, or if it is the last value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let mut x = rearr![1, 2, 3, 4];
    /// assert_eq!(x.swap_remove(1), 2);
    /// assert_eq!(x.to_vec(), vec![1, 4, 3]);
    /// x.extend([5, 6, 7]);
    /// assert_eq!(x.to_vec(), vec![1, 4, 3, 5, 6, 7]);
    /// assert_eq!(x.swap_remove(4), 6);
    /// assert_eq!(x.to_vec(), vec![1, 4, 3, 5, 7]);
    /// ```
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        if index >= N {
            self.vec.swap_remove(index - N)
        } else {
            let last_value = self.pop().unwrap();
            self.arr[index].replace(last_value).unwrap()
        }
    }
}

impl<T: ToString, const N: usize> ReArr<T, N> {
    /// Joins the [`ReArr`] into a string with a separator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use combo_vec::rearr;
    ///
    /// let x = rearr![1, 2, 3];
    /// assert_eq!(x.join(", "), "1, 2, 3");
    /// ```
    pub fn join(&self, sep: &str) -> String {
        self.iter().enumerate().fold(String::new(), |mut s, (i, item)| {
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
        if idx < N {
            self.arr[idx].as_ref().unwrap()
        } else {
            &self.vec[idx - N]
        }
    }
}

impl<T, const N: usize> ops::IndexMut<usize> for ReArr<T, N> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        if idx < N {
            self.arr[idx].as_mut().unwrap()
        } else {
            &mut self.vec[idx - N]
        }
    }
}

impl<T, const N: usize> IntoIterator for ReArr<T, N> {
    type Item = T;
    type IntoIter = Chain<Flatten<ArrayIter<Option<T>, N>>, VecIter<T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.arr.into_iter().flatten().chain(self.vec)
    }
}

impl<T> FromIterator<T> for ReArr<T, 0> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            arr: [],
            arr_len: 0,
            vec: iter.into_iter().collect(),
        }
    }
}

impl<T: Debug, const N: usize> Debug for ReArr<T, N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ReArr")
            .field("arr", &self.arr)
            .field("arr_len", &self.arr_len)
            .field("vec", &self.vec)
            .finish()
    }
}

impl<T: Debug, const N: usize> Display for ReArr<T, N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_list().entries(self.arr.iter().flatten()).entries(&self.vec).finish()
    }
}
