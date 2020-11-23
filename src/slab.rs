pub struct Slab<T: Sized> {
    data: Vec<Option<T>>,
    spaces: Vec<usize>,
}
pub struct SlabIter<'a, T> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, Option<T>>>,
}
impl<'a, T> Iterator for SlabIter<'a, T> {
    type Item = (usize, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (i, item) = self.iter.next()?;
            if let Some(item) = item {
                return Some((i, item));
            }
        }
    }
}
impl<T: Sized> Slab<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            spaces: Vec::new(),
        }
    }
    pub fn insert(&mut self, item: T) -> usize {
        if let Some(space) = self.spaces.pop() {
            self.data[space] = Some(item);
            space
        } else {
            let space = self.data.len();
            self.data.push(Some(item));
            space
        }
    }
    pub fn get_mut(&mut self, space: usize) -> Option<&mut T> {
        self.data
            .get_mut(space)
            .and_then(|x| if let Some(item) = x { Some(item) } else { None })
    }
    pub fn get(&self, space: usize) -> Option<&T> {
        self.data
            .get(space)
            .and_then(|x| if let Some(item) = x { Some(item) } else { None })
    }
    pub fn remove(&mut self, space: usize) -> Option<T> {
        if let Some(position) = self.data.get_mut(space) {
            self.spaces.push(space);
            return position.take();
        }
        None
    }
    pub fn len(&self) -> usize {
        self.data.len() - self.spaces.len()
    }
    pub fn total_len(&self) -> usize {
        self.data.len()
    }
    pub fn iter<'a>(&'a self) -> SlabIter<'a, T> {
        SlabIter {
            iter: self.data.iter().enumerate(),
        }
    }
}
