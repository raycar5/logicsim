#[derive(Debug)]
pub struct DoubleStack<T> {
    read_stack: Vec<T>,
    write_stack: Vec<T>,
}
impl<T> DoubleStack<T> {
    pub fn new() -> Self {
        Self {
            read_stack: Default::default(),
            write_stack: Default::default(),
        }
    }
    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        self.read_stack.pop()
    }
    #[inline(always)]
    pub fn push(&mut self, v: T) {
        self.write_stack.push(v);
    }
    #[inline(always)]
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.write_stack.extend(iter)
    }
    #[inline(always)]
    pub fn swap(&mut self) {
        debug_assert!(
            self.read_stack.is_empty(),
            "Tried to swap stacks while the read stack is not empty"
        );
        std::mem::swap(&mut self.read_stack, &mut self.write_stack);
    }
    pub fn len(&self) -> usize {
        self.read_stack.len() + self.write_stack.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<T: Copy> DoubleStack<T> {
    #[inline(always)]
    pub fn extend_from_slice(&mut self, v: &[T]) {
        self.write_stack.extend_from_slice(&v)
    }
}

impl<T> Default for DoubleStack<T> {
    fn default() -> Self {
        Self::new()
    }
}
