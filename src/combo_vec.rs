use crate::ReArr;
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

/// Easy creation of a new [`ComboVec`].
///
/// ## Examples
///
/// ```rust
/// use combo_vec::{combo_vec, ComboVec};
///
/// const SOME_ITEMS: ComboVec<i8, 3> = combo_vec![1, 2, 3];
/// const MANY_ITEMS: ComboVec<u16, 90> = combo_vec![5; 90];
/// const EXTRA_ITEMS: ComboVec<&str, 5> = combo_vec!["Hello", "world", "!"; None, None];
///
/// // Infer the type and size of the ComboVec
/// const NO_STACK_F32: ComboVec<f32, 0> = combo_vec![];
///
/// // No const-initialization is needed to create a ComboVec with allocated elements on the stack
/// use std::collections::HashMap;
/// const EMPTY_HASHMAP_ALLOC: ComboVec<HashMap<&str, i32>, 3> = combo_vec![];
///
/// // Creating a new ComboVec at compile time and doing this does have performance benefits
/// let my_combo_vec = EMPTY_HASHMAP_ALLOC;
/// ```
#[macro_export]
macro_rules! combo_vec {
    () => (
        $crate::ComboVec::new()
    );
    ($elem:expr; $n:expr) => (
        $crate::ComboVec::from_arr([Some($elem); $n])
    );
    ($($x:expr),+ $(,)?) => (
        $crate::ComboVec::from_arr([$(Some($x)),+])
    );
    ($($x:expr),+; $($rest:expr),* $(,)?) => (
        $crate::ComboVec::from_arr_and_len(&[$(Some($x)),+, $($rest),*])
    );
}

/// An array that can be resized at runtime, without moving any data off the stack.
///
/// Create a new [`ComboVec`] using the [`combo_vec!`] macro.
///
/// ## Examples
///
/// ```rust
/// use combo_vec::{combo_vec, ComboVec};
///
/// const SOME_ITEMS: ComboVec<i8, 3> = combo_vec![1, 2, 3];
/// const MANY_ITEMS: ComboVec<u16, 90> = combo_vec![5; 90];
///
/// // Infer the type and size of the ComboVec
/// const NO_STACK_F32: ComboVec<f32, 0> = combo_vec![];
///
/// // No const-initialization is needed to create a ComboVec with allocated elements on the stack
/// use std::collections::HashMap;
/// const EMPTY_HASHMAP_ALLOC: ComboVec<HashMap<&str, i32>, 3> = combo_vec![];
///
/// let mut my_combo_vec = combo_vec![1, 2, 3];
/// // Allocate an extra element on the heap
/// my_combo_vec.push(4);
/// assert_eq!(my_combo_vec.len(), 4);
/// // Truncate to only the first 2 elements
/// my_combo_vec.truncate(2);
/// assert_eq!(my_combo_vec.len(), 2);
/// // Fill the last element on the stack, then allocate the next two items on the heap
/// my_combo_vec.extend([3, 4, 5]);
/// ```
pub struct ComboVec<T, const N: usize> {
    arr: ReArr<T, N>,
    vec: Vec<T>,
}

impl<T: Clone, const N: usize> Clone for ComboVec<T, N> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            arr: self.arr.clone(),
            vec: self.vec.clone(),
        }
    }
}

impl<T: PartialOrd, const N: usize> PartialOrd for ComboVec<T, N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T: Ord, const N: usize> Ord for ComboVec<T, N> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T: PartialEq, const N: usize> PartialEq for ComboVec<T, N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T: PartialEq + Eq, const N: usize> Eq for ComboVec<T, N> {}

impl<T: Hash, const N: usize> Hash for ComboVec<T, N> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter().for_each(|x| x.hash(state));
    }
}

impl<T: Default, const N: usize> Default for ComboVec<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const N: usize> ComboVec<T, N> {
    /// Create a [`ComboVec`] from a fixed size array.
    ///
    /// All slots must be populated with `Some` values until
    /// the first `None` value is encountered, or the end of the array is reached.
    /// After that, all remaining slots must be `None`.
    ///
    /// This function is forced to accept a reference to the array and then copy it
    /// due to <https://github.com/rust-lang/rust/issues/80384>
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{combo_vec, ComboVec};
    ///
    /// let my_combo_vec = ComboVec::from_arr_and_len(&[Some(1), Some(2), Some(3), None, None]);
    /// let convenient_combo_vec = combo_vec![1, 2, 3; None, None];
    ///
    /// assert_eq!(my_combo_vec, convenient_combo_vec);
    /// assert_eq!(my_combo_vec.len(), 3);
    /// assert_eq!(my_combo_vec.stack_capacity(), 5);
    /// assert_eq!(my_combo_vec.heap_capacity(), 0);
    /// assert_eq!(my_combo_vec.capacity(), 5);
    /// ```
    pub const fn from_arr_and_len(arr: &[Option<T>; N]) -> Self {
        Self {
            arr: ReArr::from_arr_and_len(arr),
            vec: Vec::new(),
        }
    }
}

impl<T, const N: usize> ComboVec<T, N> {
    /// Create a new, empty [`ComboVec`] with the ability for `N` element to be allocated on the stack.
    ///
    /// This is used by the [`combo_vec!`] macro, and you should consider using it instead.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::{combo_vec, ComboVec};
    ///
    /// const COMBO_VEC: ComboVec::<i32, 3> = combo_vec![];
    /// let my_combo_vec = ComboVec::<i32, 3>::new();
    /// assert_eq!(my_combo_vec, COMBO_VEC);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            arr: ReArr::new(),
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
    /// use combo_vec::combo_vec;
    ///
    /// let mut my_combo_vec = combo_vec![1, 2, 3];
    /// let my_combo_vec2 = combo_vec![4, 5, 6];
    ///
    /// // How much space can be stored in my_combo_vec currently (length of stack + number of items in heap)
    /// // This is needed because if the stack isn't full and can store all our items, we don't want to allocate more space on the heap.
    /// let my_combo_vec_capacity = my_combo_vec2.stack_capacity() + my_combo_vec.heap_len();
    /// let extra_capacity = my_combo_vec_capacity - my_combo_vec.len();
    ///
    /// // Check if we need to reallocate
    /// if extra_capacity < my_combo_vec2.len() {
    ///     my_combo_vec.reserve(my_combo_vec2.len() - extra_capacity);
    /// }
    ///
    /// assert!(my_combo_vec.capacity() >= my_combo_vec.len() + my_combo_vec2.len());
    ///
    /// // Extend my_combo_vec with my_combo_vec2
    /// my_combo_vec.extend(my_combo_vec2);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
    }

    /// Create a [`ComboVec`] from a fixed size array.
    ///
    /// Only Some are allowed, no unitialized None values.
    ///
    /// This is used by the [`combo_vec!`] macro.
    ///
    /// ```rust
    /// use combo_vec::{combo_vec, ComboVec};
    ///
    /// let my_combo_vec = ComboVec::from_arr([Some(1), Some(2), Some(3)]);
    /// let convient_combo_vec = combo_vec![1, 2, 3];
    ///
    /// assert_eq!(my_combo_vec, convient_combo_vec);
    /// assert_eq!(my_combo_vec.len(), 3);
    /// assert_eq!(my_combo_vec.stack_capacity(), 3);
    /// assert_eq!(my_combo_vec.heap_capacity(), 0);
    /// assert_eq!(my_combo_vec.capacity(), 3);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_arr(arr: [Option<T>; N]) -> Self {
        Self {
            arr: ReArr::from_arr(arr),
            vec: Vec::new(),
        }
    }

    /// Push an element to the end of the array.
    ///
    /// If the array is full, the element will be pushed to the heap.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut my_combo_vec = combo_vec![1, 2, 3];
    /// my_combo_vec.push(4);
    /// assert_eq!(my_combo_vec.to_vec(), vec![1, 2, 3, 4]);
    /// ```
    #[inline]
    pub fn push(&mut self, val: T) {
        if self.len() < N {
            self.arr.push(val);
        } else {
            self.vec.push(val);
        }
    }

    /// Remove the last element from the array and return it, or None if it is empty.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut my_combo_vec = combo_vec![1, 2, 3];
    /// assert_eq!(my_combo_vec.pop(), Some(3));
    /// assert_eq!(my_combo_vec.pop(), Some(2));
    /// my_combo_vec.extend(combo_vec![4, 5, 6]);
    /// assert_eq!(my_combo_vec.pop(), Some(6));
    /// assert_eq!(my_combo_vec.pop(), Some(5));
    /// assert_eq!(my_combo_vec.pop(), Some(4));
    /// assert_eq!(my_combo_vec.pop(), Some(1));
    /// assert_eq!(my_combo_vec.pop(), None);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.vec.is_empty() {
            self.arr.pop()
        } else {
            self.vec.pop()
        }
    }

    /// Get any element from the array as a reference, returning `None` if out of bounds.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let my_combo_vec = combo_vec![1, 2, 3];
    /// assert_eq!(my_combo_vec.get(0), Some(&1));
    /// assert_eq!(my_combo_vec.get(1), Some(&2));
    /// assert_eq!(my_combo_vec.get(2), Some(&3));
    /// assert_eq!(my_combo_vec.get(3), None);
    /// ```
    #[must_use]
    #[inline]
    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < N {
            self.arr.get(idx)
        } else {
            self.vec.get(idx - N)
        }
    }

    /// Get any element from the array as a mutable reference, `None` if out of bounds.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut my_combo_vec = combo_vec![1, 2, 3];
    ///
    /// if let Some(x) = my_combo_vec.get_mut(0) {
    ///     *x = 4;
    /// }
    ///
    /// assert_eq!(my_combo_vec.to_vec(), vec![4, 2, 3]);
    #[must_use]
    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < N {
            self.arr.get_mut(idx)
        } else {
            self.vec.get_mut(idx - N)
        }
    }

    /// Whether or not where are any elements allocated on the heap instead of the stack
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut my_combo_vec = combo_vec![1, 2, 3];
    /// assert_eq!(my_combo_vec.spilled(), false);
    ///
    /// my_combo_vec.push(4);
    /// assert_eq!(my_combo_vec.spilled(), true);
    /// ```
    #[inline]
    pub fn spilled(&self) -> bool {
        self.heap_len() > 0
    }

    /// How many elements are currently stored on the stack.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut my_combo_vec = combo_vec![1, 2, 3];
    /// assert_eq!(my_combo_vec.stack_len(), 3);
    /// my_combo_vec.push(4);
    /// assert_eq!(my_combo_vec.stack_len(), 3);
    /// ```
    #[inline]
    pub const fn stack_len(&self) -> usize {
        self.arr.len()
    }

    /// How many elements are currently stored on the heap.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut my_combo_vec = combo_vec![1, 2, 3];
    /// assert_eq!(my_combo_vec.heap_len(), 0);
    /// my_combo_vec.push(4);
    /// assert_eq!(my_combo_vec.heap_len(), 1);
    /// ```
    #[inline]
    pub fn heap_len(&self) -> usize {
        self.vec.len()
    }

    /// How many elements are currently stored.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut my_combo_vec = combo_vec![1, 2, 3];
    /// assert_eq!(my_combo_vec.len(), 3);
    /// my_combo_vec.push(4);
    /// assert_eq!(my_combo_vec.len(), 4);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.stack_len() + self.heap_len()
    }

    /// How many elements can be stored on the stack.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::ComboVec;
    ///
    /// let mut my_combo_vec = ComboVec::<i32, 3>::new();
    /// assert_eq!(my_combo_vec.len(), 0);
    /// assert_eq!(my_combo_vec.stack_capacity(), 3);
    /// ```
    #[inline]
    pub const fn stack_capacity(&self) -> usize {
        N
    }

    /// How many elements can be stored on the currently allocated heap.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::ComboVec;
    ///
    /// let mut my_combo_vec = ComboVec::<i32, 3>::new();
    /// assert_eq!(my_combo_vec.len(), 0);
    /// assert_eq!(my_combo_vec.heap_capacity(), 0);
    /// ```
    #[inline]
    pub fn heap_capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// How many elements can be stored without reallocating anything.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::ComboVec;
    ///
    /// let mut my_combo_vec = ComboVec::<i32, 3>::new();
    /// assert_eq!(my_combo_vec.len(), 0);
    /// assert_eq!(my_combo_vec.capacity(), 3);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.stack_capacity() + self.heap_capacity()
    }

    /// Reduce the number of elements to the given length.
    ///
    /// If `new_len` is greater than the current length, this has no effect.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut my_combo_vec = combo_vec![1, 2, 3, 4, 5];
    /// my_combo_vec.truncate(3);
    /// assert_eq!(my_combo_vec.to_vec(), vec![1, 2, 3]);
    /// ```
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len > self.len() {
            // do nothing
        } else if len >= N {
            self.vec.truncate(len - N);
        } else {
            self.arr.truncate(len);
            self.vec.clear();
        }
    }

    /// Remove all elements from the array.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut my_combo_vec = combo_vec![1, 2, 3, 4, 5];
    /// my_combo_vec.clear();
    /// assert!(my_combo_vec.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.arr.clear();
        self.vec.clear();
    }

    /// Get the first element, returning `None` if there are no elements.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let my_combo_vec = combo_vec![1, 2, 3];
    /// assert_eq!(my_combo_vec.first(), Some(&1));
    /// ```
    #[inline]
    pub fn first(&self) -> Option<&T> {
        if N == 0 {
            self.vec.first()
        } else {
            self.arr.first()
        }
    }

    /// Get the first element as a mutable reference, returning `None` if there are no elements.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut my_combo_vec = combo_vec![1, 2, 3];
    /// *my_combo_vec.first_mut().unwrap() = 4;
    /// assert_eq!(my_combo_vec.first(), Some(&4));
    /// ```
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut T> {
        if N == 0 {
            self.vec.first_mut()
        } else {
            self.arr.first_mut()
        }
    }

    /// Get the last element, returning `None` if there are no elements.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let my_combo_vec = combo_vec![1, 2, 3];
    /// assert_eq!(my_combo_vec.last(), Some(&3));
    /// ```
    #[inline]
    pub fn last(&self) -> Option<&T> {
        if N == 0 {
            self.vec.last()
        } else {
            self.vec.last().or_else(|| self.arr.last())
        }
    }

    /// Get the last element as a mutable reference, returning `None` if there are no elements.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut my_combo_vec = combo_vec![1, 2, 3];
    /// *my_combo_vec.last_mut().unwrap() = 4;
    /// assert_eq!(my_combo_vec.last(), Some(&4));
    /// ```
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if N == 0 {
            self.vec.last_mut()
        } else {
            self.vec.last_mut().or_else(|| self.arr.last_mut())
        }
    }

    /// Check if there are no elements.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::ComboVec;
    ///
    /// let mut my_combo_vec = ComboVec::<i32, 3>::new();
    /// assert!(my_combo_vec.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an iterator over the elements of the array.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let x = combo_vec![1, 2, 3];
    /// let mut iter = x.iter();
    ///
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), Some(&3));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.arr.iter().chain(self.vec.iter())
    }

    /// Get an iterator over the elements of the array, returning mutable references.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut x = combo_vec![1, 2, 3];
    /// for i in x.iter_mut() {
    ///    *i += 1;
    /// }
    /// assert_eq!(x.to_vec(), vec![2, 3, 4]);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.arr.iter_mut().chain(self.vec.iter_mut())
    }

    /// Extend this array with all the elements from the given iterator.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut x = combo_vec![1, 2, 3];
    /// x.extend(vec![4, 5, 6]);
    /// assert_eq!(x.to_vec(), vec![1, 2, 3, 4, 5, 6]);
    /// ```
    #[inline]
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|x| self.push(x));
    }

    /// Get this [`ComboVec`] transformed into a [`Vec`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let x = combo_vec![1, 2, 3];
    /// assert_eq!(x.into_vec(), vec![1, 2, 3]);
    /// ```
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.into_iter().collect()
    }

    /// Get this [`ComboVec`] represented as a [`Vec`], borrowing data instead of moving it.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let x = combo_vec![1, 2, 3];
    /// assert_eq!(x.ref_vec(), vec![&1, &2, &3]);
    /// ```
    #[inline]
    pub fn ref_vec(&self) -> Vec<&T> {
        self.iter().collect()
    }
}

impl<T: Clone, const N: usize> ComboVec<T, N> {
    /// Get this [`ComboVec`] represented as a [`Vec`].
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let x = combo_vec![1, 2, 3];
    /// assert_eq!(x.to_vec(), vec![1, 2, 3]);
    /// ```
    #[inline]
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

    /// Resizes the [`ComboVec`] in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the [`ComboVec`] is extended by the
    /// difference, with each additional slot filled with `val`.
    ///
    /// If `new_len` is less than `len`, the [`ComboVec`] is truncated.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut x = combo_vec![1, 2, 3];
    /// x.resize(5, 4);
    /// assert_eq!(x.to_vec(), vec![1, 2, 3, 4, 4]);
    /// x.resize(2, 5);
    /// assert_eq!(x.to_vec(), vec![1, 2]);
    /// x.resize(5, 6);
    /// assert_eq!(x.to_vec(), vec![1, 2, 6, 6, 6]);
    /// ```
    pub fn resize(&mut self, new_len: usize, val: T) {
        if new_len >= N {
            if self.len() < N {
                self.arr.resize(N, val.clone());
            }

            self.vec.resize(new_len - N, val);
        } else {
            self.arr.resize(new_len, val);
            self.vec.clear();
        }
    }

    /// Resizes the [`ComboVec`] in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the [`ComboVec`] is extended by the
    /// difference, with each additional slot filled with the result of calling
    /// the closure `f`.
    ///
    /// If `new_len` is less than `len`, the [`ComboVec`] is truncated.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let mut x = combo_vec![1, 2, 3];
    /// x.resize_with(5, Default::default);
    /// assert_eq!(x.to_vec(), vec![1, 2, 3, 0, 0]);
    /// x.resize_with(2, Default::default);
    /// assert_eq!(x.to_vec(), vec![1, 2]);
    /// x.resize_with(5, Default::default);
    /// assert_eq!(x.to_vec(), vec![1, 2, 0, 0, 0]);
    /// ```
    pub fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, mut f: F) {
        if new_len >= N {
            for _ in self.len()..N {
                self.arr.push(f());
            }

            self.vec.resize_with(new_len - N, f);
        } else {
            self.arr.resize_with(new_len, f);
            self.vec.clear();
        }
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
    /// use combo_vec::combo_vec;
    ///
    /// let mut x = combo_vec![1, 2, 3];
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
            let val = self.arr.remove(index);

            if !self.vec.is_empty() {
                self.arr.push(self.vec.remove(0));
            }

            val
        }
    }

    /// Removes an element from the `ComboVec` and returns it.
    ///
    /// The removed element is replaced by the last element of the `ComboVec`.
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
    /// use combo_vec::combo_vec;
    ///
    /// let mut x = combo_vec![1, 2, 3, 4];
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
        } else if self.len() <= N {
            self.arr.swap_remove(index)
        } else {
            let last_value = self.vec.pop().unwrap();
            // optimization that requires we reach into
            // the underlying representation of the array
            self.arr.arr[index].replace(last_value).unwrap()
        }
    }
}

impl<T: ToString, const N: usize> ComboVec<T, N> {
    /// Joins the [`ComboVec`] into a string with a separator.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use combo_vec::combo_vec;
    ///
    /// let x = combo_vec![1, 2, 3];
    /// assert_eq!(x.join(", "), "1, 2, 3");
    /// ```
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

impl<T, const N: usize> ops::Index<usize> for ComboVec<T, N> {
    type Output = T;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        if idx < N {
            &self.arr[idx]
        } else {
            &self.vec[idx - N]
        }
    }
}

impl<T, const N: usize> ops::IndexMut<usize> for ComboVec<T, N> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        if idx < N {
            &mut self.arr[idx]
        } else {
            &mut self.vec[idx - N]
        }
    }
}

impl<T, const N: usize> IntoIterator for ComboVec<T, N> {
    type Item = T;
    type IntoIter = Chain<Flatten<ArrayIter<Option<T>, N>>, VecIter<T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.arr.into_iter().chain(self.vec)
    }
}

impl<T> FromIterator<T> for ComboVec<T, 0> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            arr: ReArr::new(),
            vec: iter.into_iter().collect(),
        }
    }
}

impl<T: Debug, const N: usize> Debug for ComboVec<T, N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ComboVec")
            .field("arr", &self.arr)
            .field("vec", &self.vec)
            .finish()
    }
}

impl<T: Debug, const N: usize> Display for ComboVec<T, N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_list().entries(self.arr.iter()).entries(&self.vec).finish()
    }
}
