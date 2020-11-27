use std::ops::Deref;

/// Data structure that enforces immutability at compile time.
///
/// It implements [Deref] so all operations on the underlying type will
/// work as normal, as long as they take an immutable reference.
///
/// # Examples
///
/// This does not compile:
/// ```compile_fail
/// # use logicsim::data_structures::Immutable;
/// let v = Immutable::new(vec![1,2]);
///
/// v.push(2);
///
/// ```
///
/// This does compile:
/// ```
/// # use logicsim::data_structures::Immutable;
/// let v = Immutable::new(vec![1,2]);
///
/// assert_eq!(v[0], 1);
///
/// ```
#[repr(transparent)]
pub struct Immutable<T>(T);
impl<T> Immutable<T> {
    /// Returns a new [Immutable] containing `value`.
    pub fn new(value: T) -> Self {
        Self(value)
    }
    #[inline(always)]
    fn get_immutable(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for Immutable<T> {
    fn from(i: T) -> Self {
        Self(i)
    }
}

impl<T> Deref for Immutable<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get_immutable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO add failing test if this goes anywhere.
    // https://github.com/rust-lang/rust/issues/12335.
    #[test]
    fn test_immutable() {
        let i = Immutable::new(vec![1, 2, 3]);

        assert_eq!(i[2], 3);
    }
}
