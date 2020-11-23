#[derive(Debug)]
pub struct DoubleStack<T> {
    stacks: [Vec<T>; 2],
    read_stack_is_0: bool,
}
impl<T> DoubleStack<T> {
    pub fn new() -> Self {
        Self {
            stacks: Default::default(),
            read_stack_is_0: false,
        }
    }
    #[inline(always)]
    fn read_stack_index(&mut self) -> usize {
        if self.read_stack_is_0 {
            0
        } else {
            1
        }
    }
    #[inline(always)]
    fn write_stack_index(&mut self) -> usize {
        if self.read_stack_is_0 {
            1
        } else {
            0
        }
    }
    #[inline(always)]
    fn read_stack(&mut self) -> &mut Vec<T> {
        &mut self.stacks[self.read_stack_index()]
    }
    #[inline(always)]
    fn write_stack(&mut self) -> &mut Vec<T> {
        &mut self.stacks[self.write_stack_index()]
    }
    pub fn pop(&mut self) -> Option<T> {
        self.read_stack().pop()
    }
    pub fn push(&mut self, v: T) {
        self.write_stack().push(v);
    }
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.write_stack().extend(iter)
    }
    pub fn swap(&mut self) {
        debug_assert!(
            self.read_stack().is_empty(),
            "Tried to swap stacks while the read stack is not empty"
        );
        self.read_stack_is_0 = !self.read_stack_is_0
    }
    pub fn len(&self) -> usize {
        self.stacks[0].len() + self.stacks[1].len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Default for DoubleStack<T> {
    fn default() -> Self {
        Self::new()
    }
}
