use indexmap::IndexSet;
use std::mem::MaybeUninit;
pub struct Slab<T: Sized> {
    data: Vec<MaybeUninit<T>>,
    empty_spaces: IndexSet<usize>,
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
    pub fn total_len(&self) -> usize {
        self.data.len()
    }
    pub fn iter<'a>(&'a self) -> SlabIter<'a, T> {
        SlabIter {
            iter: self.data.iter().enumerate(),
            empty_spaces: &self.empty_spaces,
        }
    }
}
