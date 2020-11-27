use std::iter::FromIterator;

/// Data structure consisting of a write stack and a read stack, write operations are performed on the write stack,
/// read operations are performed on the read stack and calling [DoubleStack::swap] swaps them.
///
/// # Example
/// ```
/// # use logicsim::data_structures::DoubleStack;
/// let mut stacks = DoubleStack::new();
///
/// stacks.push(1);
/// stacks.push(2);
/// stacks.push(3);
///
/// assert_eq!(stacks.pop(), None);
///
/// stacks.swap();
///
/// assert_eq!(stacks.pop().unwrap(), 3);
/// assert_eq!(stacks.pop().unwrap(), 2);
/// assert_eq!(stacks.pop().unwrap(), 1);
///
/// assert_eq!(stacks.pop(), None);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DoubleStack<T> {
    read_stack: Vec<T>,
    write_stack: Vec<T>,
}

impl<T> DoubleStack<T> {
    /// Returns an empty [DoubleStack].
    pub fn new() -> Self {
        Self {
            read_stack: Default::default(),
            write_stack: Default::default(),
        }
    }

    /// Pops an item from the end of the read stack and returns it.
    /// If the read stack is empty, returns None.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        self.read_stack.pop()
    }

    /// Pushes an item to the end of the write stack.
    #[inline(always)]
    pub fn push(&mut self, v: T) {
        self.write_stack.push(v);
    }

    /// Pushes all the items in the iterator to the end of the write stack.
    #[inline(always)]
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.write_stack.extend(iter)
    }

    /// Swaps the write and read stacks, after calling this method you can [pop](DoubleStack::pop)
    /// items that you had previously [pushed](DoubleStack::push).
    #[inline(always)]
    pub fn swap(&mut self) {
        debug_assert!(
            self.read_stack.is_empty(),
            "Tried to swap stacks while the read stack is not empty"
        );
        std::mem::swap(&mut self.read_stack, &mut self.write_stack);
    }

    /// Returns the sum of the items in the read and write stacks.
    pub fn len(&self) -> usize {
        self.read_stack.len() + self.write_stack.len()
    }

    /// Returns true if both the read and write stacks are empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Clone> DoubleStack<T> {
    #[inline(always)]
    /// Clones all items from the slice to the end of the write stack.
    pub fn extend_from_slice(&mut self, v: &[T]) {
        self.write_stack.extend_from_slice(&v)
    }
}

impl<T> Default for DoubleStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FromIterator<T> for DoubleStack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            read_stack: Default::default(),
            write_stack: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut s: DoubleStack<u8> = Default::default();

        assert_eq!(s.pop(), None);

        for i in 0..10 {
            s.push(i);
            assert_eq!(s.pop(), None);
        }

        s.swap();

        for i in (0..10).rev() {
            s.push(i);
            assert_eq!(s.pop(), Some(i));
        }
        assert_eq!(s.pop(), None);

        s.swap();

        for i in 0..10 {
            assert_eq!(s.pop(), Some(i));
        }
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn test_extend() {
        let mut s: DoubleStack<u8> = Default::default();

        s.extend(0..10);
        assert_eq!(s.pop(), None);

        s.swap();
        for i in (0..10).rev() {
            assert_eq!(s.pop(), Some(i))
        }
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn test_extend_from_slice() {
        let mut s: DoubleStack<u8> = Default::default();

        s.extend(0..10);
        assert_eq!(s.pop(), None);

        s.swap();
        for i in (0..10).rev() {
            assert_eq!(s.pop(), Some(i))
        }
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn test_from_iter() {
        let mut s: DoubleStack<u8> = (0..10).collect();

        assert_eq!(s.pop(), None);

        s.swap();
        for i in (0..10).rev() {
            assert_eq!(s.pop(), Some(i))
        }
        assert_eq!(s.pop(), None);
    }
}
