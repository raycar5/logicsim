use super::SlabIndex;
use indexmap::IndexSet;
use std::mem::MaybeUninit;

/// Simple slab allocator. Stores items of the same type and can reuse removed indexes.
///
/// # Example
///
/// ```
/// # use logicsim::data_structures::Slab;
/// let mut s = Slab::new();
///
/// let index = s.insert(5);
/// assert_eq!(s.get(index), Some(&5));
///
/// assert_eq!(s.remove(index), Some(5));
///
/// assert_eq!(s.get(index), None);
/// ```
#[derive(Debug)]
pub struct Slab<T: Sized> {
    data: Vec<MaybeUninit<T>>,
    removed_indexes: IndexSet<SlabIndex>,
}
impl<T: Sized> Slab<T> {
    /// Returns an empty [Slab].
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            removed_indexes: IndexSet::new(),
        }
    }

    /// Inserts an item into the slab and returns its index.
    ///
    /// Will reuse an empty index if one is available.
    pub fn insert(&mut self, item: T) -> SlabIndex {
        if let Some(index) = self.removed_indexes.pop() {
            self.data[index.0] = MaybeUninit::new(item);
            index
        } else {
            let index = SlabIndex(self.data.len());
            self.data.push(MaybeUninit::new(item));
            index
        }
    }

    /// Returns a mutable reference to the item at `index`.
    ///
    /// Returns [None] if `index` has been removed.
    pub fn get_mut(&mut self, index: SlabIndex) -> Option<&mut T> {
        if let Some(item) = self.data.get_mut(index.0) {
            if self.removed_indexes.contains(&index) {
                return None;
            }
            // This is safe because we check if the item is an empty space.
            unsafe { return Some(item.assume_init_mut()) };
        }
        None
    }

    /// Return a reference to the item at `index`.
    ///
    /// Returns [None] if `index` has been removed.
    pub fn get(&self, index: SlabIndex) -> Option<&T> {
        if let Some(item) = self.data.get(index.0) {
            if self.removed_indexes.contains(&index) {
                return None;
            }
            // This is safe because we check if the item is an empty space.
            unsafe { return Some(item.assume_init_ref()) };
        }
        None
    }

    /// Removes an item from the Slab and returns it.
    ///
    /// Returns [None] if `index` has been removed.
    /// `index` will be reused on the next call to [Slab::insert].
    // All the safety in this data structure depends on the implementation of this method.
    pub fn remove(&mut self, index: SlabIndex) -> Option<T> {
        if let Some(position) = self.data.get_mut(index.0) {
            if self.removed_indexes.contains(&index) {
                return None;
            }
            self.removed_indexes.insert(index);
            let item = std::mem::replace(position, MaybeUninit::uninit());
            // This is safe because we check if the item is an empty space.
            unsafe { return Some(item.assume_init()) };
        }
        None
    }

    /// Returns the number of items in the slab.
    ///
    /// This is different from the number of allocated slots in the slab, see [Slab::total_len]
    pub fn len(&self) -> usize {
        self.data.len() - self.removed_indexes.len()
    }

    /// Returns true if the number of items in the slab is 0.
    ///
    /// This is different from the number of allocated slots in the slab, see [Slab::total_len]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of allocated slots in the slab, some of them could be empty.
    pub fn total_len(&self) -> usize {
        self.data.len()
    }

    /// Returns an iterator over pairs of ```(SlabIndex, [&T])```.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            iter: self.data.iter().enumerate(),
            removed_indexes: &self.removed_indexes,
        }
    }

    /// Returns the item at index without performing bounds checking or checking if the slot contains initialized data.
    ///
    /// # Safety
    /// This function is safe if `index` < [Slab::total_len()]
    /// and the item at `index` has not been removed.
    /// Will panic in debug mode if the invariants are broken.
    ///
    /// Annoyingly long names discourage use and make you really think about what you are doing.
    pub unsafe fn get_very_unsafely(&self, index: SlabIndex) -> &T {
        debug_assert!(
            index.0 < self.data.len(),
            "Tried to access index out of bounds, len:{}, index:{}",
            self.data.len(),
            index
        );
        debug_assert!(
            !self.removed_indexes.contains(&index),
            "Tried to access removed index:{}",
            index
        );
        self.data.get_unchecked(index.0).assume_init_ref()
    }
}

impl<T: Clone> Clone for Slab<T> {
    fn clone(&self) -> Self {
        let mut data = Vec::new();
        data.reserve(self.data.len());
        for (index, item) in self.data.iter().enumerate() {
            if self.removed_indexes.contains(&SlabIndex(index)) {
                data.push(MaybeUninit::uninit());
            } else {
                // This is safe because we check that the item is not an empty space.
                unsafe { data.push(MaybeUninit::new(item.assume_init_ref().clone())) };
            }
        }

        Self {
            data,
            removed_indexes: self.removed_indexes.clone(),
        }
    }
}

/// [IntoIterator] for [Slab]
pub struct IntoIter<T> {
    slab: Slab<T>,
    i: SlabIndex,
}
impl<T> IntoIterator for Slab<T> {
    type IntoIter = IntoIter<T>;
    type Item = (SlabIndex, T);
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            slab: self,
            i: SlabIndex(0),
        }
    }
}
impl<T> Iterator for IntoIter<T> {
    type Item = (SlabIndex, T);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.i.0 == self.slab.data.len() {
                return None;
            }
            if self.slab.removed_indexes.contains(&self.i) {
                self.i.0 += 1;
                continue;
            }
            // This is safe because we check if the item is an empty space.
            let item = unsafe {
                Some((
                    self.i,
                    std::mem::replace(&mut self.slab.data[self.i.0], MaybeUninit::uninit())
                        .assume_init(),
                ))
            };
            self.i.0 += 1;
            return item;
        }
    }
}

/// [Iterator] for [Slab]
pub struct Iter<'a, T> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, MaybeUninit<T>>>,
    removed_indexes: &'a IndexSet<SlabIndex>,
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (SlabIndex, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (i, item) = self.iter.next()?;
            let si = SlabIndex(i);

            if self.removed_indexes.contains(&si) {
                continue;
            }

            // This is safe because we check if the item is an empty space.
            unsafe { return Some((si, item.assume_init_ref())) };
        }
    }
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut s: Slab<_> = Default::default();

        assert_eq!(s.get(SlabIndex(0)), None);

        let index = s.insert(1);
        assert_eq!(*s.get(index).unwrap(), 1);
        assert_eq!(s.get(SlabIndex(1)), None);

        s.remove(index);
        assert_eq!(s.get(index), None);
    }

    #[test]
    fn test_get_mut() {
        let mut s: Slab<_> = Default::default();

        assert_eq!(s.get_mut(SlabIndex(0)), None);

        let index = s.insert(1);
        assert_eq!(*s.get_mut(index).unwrap(), 1);
        assert_eq!(s.get_mut(SlabIndex(1)), None);

        s.remove(index);
        assert_eq!(s.get_mut(index), None);
    }

    #[test]
    fn test_remove() {
        let mut s = Slab::new();

        assert_eq!(s.remove(SlabIndex(0)), None);

        let index = s.insert(1);
        assert_eq!(s.remove(index), Some(1));

        let new_index = s.insert(2);
        assert_eq!(index, new_index);
        assert_eq!(s.remove(new_index), Some(2));
    }

    #[test]
    fn test_len() {
        let mut s = Slab::new();

        assert_eq!(s.len(), 0);
        assert_eq!(s.is_empty(), true);
        assert_eq!(s.total_len(), 0);

        let index = s.insert(1);
        assert_eq!(s.len(), 1);
        assert_eq!(s.is_empty(), false);
        assert_eq!(s.total_len(), 1);

        s.remove(index);
        assert_eq!(s.len(), 0);
        assert_eq!(s.is_empty(), true);
        assert_eq!(s.total_len(), 1);
    }

    #[test]
    fn test_iter() {
        let mut s = Slab::new();
        for i in 0..10 {
            s.insert(i);
        }
        for i in (1..10).step_by(2) {
            s.remove(SlabIndex(i));
        }
        for (i, n) in s.iter() {
            assert_eq!(i.0, *n)
        }
    }

    #[test]
    fn test_into_iter() {
        let mut s = Slab::new();
        for i in 0..10 {
            s.insert(i);
        }
        for i in (0..10).step_by(2) {
            s.remove(SlabIndex(i));
        }
        for (i, n) in s.into_iter() {
            assert_eq!(i.0, n)
        }
    }

    #[test]
    fn test_clone() {
        let mut s = Slab::new();
        for i in 0..10 {
            s.insert(i);
        }
        for i in (0..10).step_by(2) {
            s.remove(SlabIndex(i));
        }
        let ss = s.clone();
        for ((i1, n1), (i2, n2)) in s.into_iter().zip(ss) {
            assert_eq!(i1, i2);
            assert_eq!(n1, n2);
        }
    }
    #[test]
    fn test_get_very_unsafely() {
        let mut s = Slab::new();

        let index = s.insert(1);
        let other_index = s.insert(2);

        s.remove(index);
        unsafe { assert_eq!(*s.get_very_unsafely(other_index), 2) };
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "Tried to access index out of bounds, len:0, index:0")]
    fn test_get_very_unsafely_panics_out_of_bounds() {
        let s = Slab::<u8>::new();

        unsafe { assert_eq!(*s.get_very_unsafely(SlabIndex(0)), 2) };
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "Tried to access removed index:1")]
    fn test_get_very_unsafely_panics_removed_index() {
        let mut s = Slab::<u8>::new();

        s.insert(3);
        let index = s.insert(2);
        s.insert(4);

        s.remove(index);

        unsafe { assert_eq!(*s.get_very_unsafely(index), 2) };
    }
}
