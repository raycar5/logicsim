pub struct Slab<T: Sized> {
    data: Vec<Option<T>>,
    spaces: Vec<usize>,
}
impl<T: Sized> Slab<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            spaces: Vec::new(),
        }
    }
    pub fn insert(&mut self, item: T) -> Option<usize> {
        Some(if let Some(space) = self.spaces.pop() {
            self.data[space] = Some(item);
            space
        } else {
            let space = self.data.len();
            self.data.push(Some(item));
            space
        })
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
    pub fn len(&self) -> usize {
        self.data.len() - self.spaces.len()
    }
}
