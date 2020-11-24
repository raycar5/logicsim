use indexmap::IndexSet;
use std::mem::MaybeUninit;
#[derive(Debug)]
pub struct Slab<T: Sized> {
    data: Vec<MaybeUninit<T>>,
    empty_spaces: IndexSet<usize>,
}
impl<T: Clone> Clone for Slab<T> {
    fn clone(&self) -> Self {
        let mut data = Vec::new();
        data.reserve(self.data.len());
        for (i, item) in self.data.iter().enumerate() {
            if self.empty_spaces.contains(&i) {
                data.push(MaybeUninit::uninit());
            } else {
                // This is safe because we check that the item is not an empty space.
                unsafe { data.push(MaybeUninit::new(item.assume_init_ref().clone())) };
            }
        }

        Self {
            data,
            empty_spaces: self.empty_spaces.clone(),
        }
    }
}
impl<T> IntoIterator for Slab<T> {
    type IntoIter = SlabIntoIter<T>;
    type Item = (usize, T);
    fn into_iter(self) -> Self::IntoIter {
        SlabIntoIter { slab: self, i: 0 }
    }
}
pub struct SlabIntoIter<T> {
    slab: Slab<T>,
    i: usize,
}
impl<T> Iterator for SlabIntoIter<T> {
    type Item = (usize, T);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.i == self.slab.data.len() {
                return None;
            }
            if self.slab.empty_spaces.contains(&self.i) {
                self.i += 1;
                continue;
            }
            // This is safe because we check if the item is an empty space.
            let item = unsafe {
                Some((
                    self.i,
                    std::mem::replace(&mut self.slab.data[self.i], MaybeUninit::uninit())
                        .assume_init(),
                ))
            };
            self.i += 1;
            return item;
        }
    }
}

pub struct SlabIter<'a, T> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, MaybeUninit<T>>>,
    empty_spaces: &'a IndexSet<usize>,
}
impl<'a, T> Iterator for SlabIter<'a, T> {
    type Item = (usize, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (i, item) = self.iter.next()?;
            if self.empty_spaces.contains(&i) {
                continue;
            }
            // This is safe because we check if the item is an empty space.
            unsafe { return Some((i, item.assume_init_ref())) };
        }
    }
}

impl<T: Sized> Slab<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            empty_spaces: IndexSet::new(),
        }
    }
    pub fn insert(&mut self, item: T) -> usize {
        if let Some(space) = self.empty_spaces.pop() {
            self.data[space] = MaybeUninit::new(item);
            space
        } else {
            let space = self.data.len();
            self.data.push(MaybeUninit::new(item));
            space
        }
    }
    pub fn get_mut(&mut self, space: usize) -> Option<&mut T> {
        if let Some(item) = self.data.get_mut(space) {
            if self.empty_spaces.contains(&space) {
                return None;
            }
            // This is safe because we check if the item is an empty space.
            unsafe { return Some(item.assume_init_mut()) };
        }
        None
    }
    /// # Safety
    /// This function is safe as long as space < [Slab::total_len()]
    /// and the item at space has not been removed.
    /// This invariants are checked in debug mode.
    pub unsafe fn get_very_unsafely(&self, space: usize) -> &T {
        debug_assert!(space < self.data.len());
        debug_assert!(!self.empty_spaces.contains(&space));
        self.data.get_unchecked(space).assume_init_ref()
    }
    pub fn get(&self, space: usize) -> Option<&T> {
        if let Some(item) = self.data.get(space) {
            if self.empty_spaces.contains(&space) {
                return None;
            }
            // This is safe because we check if the item is an empty space.
            unsafe { return Some(item.assume_init_ref()) };
        }
        None
    }
    // All the safety in this data structure depends on the implementation of this method.
    pub fn remove(&mut self, space: usize) -> Option<T> {
        if let Some(position) = self.data.get_mut(space) {
            if self.empty_spaces.contains(&space) {
                return None;
            }
            self.empty_spaces.insert(space);
            let item = std::mem::replace(position, MaybeUninit::uninit());
            // This is safe because we check if the item is an empty space.
            unsafe { return Some(item.assume_init()) };
        }
        None
    }
    pub fn len(&self) -> usize {
        self.data.len() - self.empty_spaces.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn total_len(&self) -> usize {
        self.data.len()
    }
    pub fn iter(&self) -> SlabIter<T> {
        SlabIter {
            iter: self.data.iter().enumerate(),
            empty_spaces: &self.empty_spaces,
        }
    }
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self::new()
    }
}
