#[repr(transparent)]
pub struct Immutable<T>(T);
impl<T> Immutable<T> {
    pub fn new(i: T) -> Self {
        Self(i)
    }
    #[inline(always)]
    pub fn get(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for Immutable<T> {
    fn from(i: T) -> Self {
        Self(i)
    }
}
