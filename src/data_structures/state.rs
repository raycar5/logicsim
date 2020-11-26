use super::word_mask_64;
use num_integer::div_ceil;
use unwrap::unwrap;

/// Data structure that represents a fixed size (at runtime) array of bits,
/// [State] will keep track of when bits are updated until the next call to [State::tick].
///
/// State will allocate bits in multiples of 64.
/// # Example
/// ```
/// # use wires::data_structures::State;
/// let mut s = State::new(2);
///
/// assert_eq!(s.len(), 64);
///
/// s.set(1,true);
/// assert_eq!(s.get_state(1), true);
/// assert_eq!(s.get_updated(1), true);
///
/// s.tick();
/// assert_eq!(s.get_state(1), true);
/// assert_eq!(s.get_updated(1), false);
/// ```
///
/// # Panics
///
/// Panics if you try to read or write to an index >= [State::len()]
///
/// ```should_panic
/// # use wires::data_structures::State;
/// let mut s = State::new(2);
///
/// s.get_state(64);
/// ```
///
///
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct State {
    states: Vec<u64>,
    updated: Vec<u64>,
}
impl State {
    /// Returns a new [State] with `n` bits all of which are initialized to `false`.
    pub fn new(n: usize) -> State {
        let states = vec![0; div_ceil(n, 64)];
        let updated = vec![0; div_ceil(n, 64)];

        State { states, updated }
    }

    /// Returns true if the bit at `index` is 1 in vector `v`.
    ///
    /// See [super::word_mask_64] for details.
    #[inline(always)]
    fn get_bit_from_vec(v: &[u64], index: usize) -> bool {
        let (word_index, mask) = word_mask_64(index);
        let word = unwrap!(
            v.get(word_index),
            "Tried to access index out of bounds:{}, size:{}",
            index,
            v.len() * 64,
        );

        word & mask != 0
    }

    /// Returns true if the bit at `index` is set.
    ///
    /// # Panics
    ///
    /// Panics if `index` >= [State::len()]
    pub fn get_state(&self, index: usize) -> bool {
        Self::get_bit_from_vec(&self.states, index)
    }

    /// Returns true if the bit at `index` has been [set](State::set) since the last call to [State::tick].
    ///
    /// # Panics
    ///
    /// Panics if `index` >= [State::len()]
    pub fn get_updated(&self, index: usize) -> bool {
        Self::get_bit_from_vec(&self.updated, index)
    }

    /// Returns true if the bit at `index` is set.
    /// Returns None if the bit has not been [set](State::set) since the last call to [State::tick].
    ///
    /// # Panics
    ///
    /// Panics if `index` >= [State::len()]
    pub fn get_if_updated(&self, index: usize) -> Option<bool> {
        if self.get_updated(index) {
            Some(self.get_state(index))
        } else {
            None
        }
    }

    /// Sets the bit at `index` to `value` and keeps track that it has been updated.
    ///
    /// # Panics
    ///
    /// Panics if `index` >= [State::len()]
    pub fn set(&mut self, index: usize, value: bool) {
        let (word_index, mask) = word_mask_64(index);

        let state = &mut self.states[word_index];
        if value {
            *state |= mask;
        } else {
            *state &= !mask;
        }

        let updated = &mut self.updated[word_index];
        *updated |= mask;
    }

    /// Manually marks the bit at `index` as updated, this is equivalent to:
    /// ```
    /// # use wires::data_structures::State;
    /// # let mut s = State::new(2);
    /// s.set(0,s.get_state(0));
    /// ```
    /// # Panics
    ///
    /// Panics if `index` >= [State::len()]
    pub fn set_updated(&mut self, index: usize) {
        let (word_index, mask) = word_mask_64(index);
        let updated = &mut self.updated[word_index];
        *updated |= mask;
    }

    /// Resets the updated state of every bit to false.
    pub fn tick(&mut self) {
        for updated in &mut self.updated {
            *updated = 0
        }
    }

    /// Returns the number of bits in the [State].
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.states.len() * 64
    }

    // The dark corner.

    /// Unsafe version of [State::get_bit_from_vec].
    ///
    /// # Safety
    /// This function is safe if real_index < v.len() .
    /// Will panic in debug mode if the invariant is broken.
    ///
    /// Annoyingly long names discourage use and make you really think about what you are doing.
    #[inline(always)]
    unsafe fn get_bit_from_vec_very_unsafely(v: &[u64], index: usize) -> bool {
        let (word_index, mask) = word_mask_64(index);
        debug_assert!(
            word_index < v.len(),
            "Tried to access index:{} >= State::len():{}",
            index,
            v.len() * 64
        );

        let word = v.get_unchecked(word_index);
        word & mask != 0
    }

    /// Unsafe version of [State::get_state].
    ///
    /// # Safety
    /// This function is safe if index < [State::len()].
    /// Will panic in debug mode if the invariant is broken.
    ///
    /// Annoyingly long names discourage use and make you really think about what you are doing.
    #[inline(always)]
    pub unsafe fn get_state_very_unsafely(&self, index: usize) -> bool {
        Self::get_bit_from_vec_very_unsafely(&self.states, index)
    }

    /// Unsafe version of [State::get_updated].
    ///
    /// # Safety
    /// This function is safe if index < [State::len()].
    /// Will panic in debug mode if the invariant is broken.
    ///
    /// Annoyingly long names discourage use and make you really think about what you are doing.
    #[inline(always)]
    pub unsafe fn get_updated_very_unsafely(&self, index: usize) -> bool {
        Self::get_bit_from_vec_very_unsafely(&self.updated, index)
    }

    /// Unsafe version of [State::get_if_updated].
    ///
    /// # Safety
    /// This function is safe if index < [State::len()].
    /// Will panic in debug mode if the invariant is broken.
    ///
    /// Annoyingly long names discourage use and make you really think about what you are doing.
    #[inline(always)]
    pub unsafe fn get_if_updated_very_unsafely(&self, index: usize) -> Option<bool> {
        if self.get_updated_very_unsafely(index) {
            Some(self.get_state_very_unsafely(index))
        } else {
            None
        }
    }

    /// Unsafe version of [State::set].
    ///
    /// # Safety
    /// This function is safe if index < [State::len()].
    /// Will panic in debug mode if the invariant is broken.
    ///
    /// Annoyingly long names discourage use and make you really think about what you are doing.
    #[inline(always)]
    pub unsafe fn set_very_unsafely(&mut self, index: usize, value: bool) {
        let (word_index, mask) = word_mask_64(index);

        debug_assert!(
            word_index < self.states.len(),
            "Tried to write to index:{} >= State::len():{}",
            index,
            self.states.len() * 64
        );
        debug_assert!(
            word_index < self.updated.len(),
            "Tried to write to index:{} >= State::len():{}",
            index,
            self.updated.len() * 64
        );

        let state = self.states.get_unchecked_mut(word_index);
        if value {
            *state |= mask;
        } else {
            *state &= !mask;
        }

        let updated = &mut self.updated[word_index];
        *updated |= mask;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_set() {
        for i in 2..100 {
            let mut state = State::new(100);
            assert_eq!(state.get_state(i), false);
            assert_eq!(state.get_updated(i), false);

            state.set(i, true);

            assert_eq!(state.get_state(i), true);
            assert_eq!(state.get_updated(i), true);

            state.set(i, false);

            assert_eq!(state.get_state(i), false);
            assert_eq!(state.get_updated(i), true);
        }
    }

    #[test]
    fn test_tick() {
        let mut state = State::new(100);
        for i in 2..100 {
            assert_eq!(state.get_state(i), false, "index: {}", i);
            assert_eq!(state.get_updated(i), false, "index: {}", i);

            state.set(i, true);

            assert_eq!(state.get_state(i), true, "index: {}", i);
            assert_eq!(state.get_updated(i), true, "index: {}", i);

            state.tick();

            assert_eq!(state.get_state(i), true, "index: {}", i);
            assert_eq!(state.get_updated(i), false, "index: {}", i);
        }
    }

    #[test]
    fn test_get_if_updated() {
        let mut state = State::new(2);

        assert_eq!(state.get_if_updated(0), None);

        state.set(0, true);
        assert_eq!(state.get_if_updated(0), Some(true));

        state.tick();
        assert_eq!(state.get_if_updated(0), None);
    }

    #[test]
    fn test_set_updated() {
        let mut state = State::new(2);

        assert_eq!(state.get_updated(0), false);

        let old_state = state.get_state(0);

        state.set_updated(0);
        assert_eq!(state.get_if_updated(0), Some(old_state));

        state.tick();
        assert_eq!(state.get_if_updated(0), None);
    }

    #[test]
    fn test_len() {
        assert_eq!(State::new(2).len(), 64);
        assert_eq!(State::new(64).len(), 64);
        assert_eq!(State::new(65).len(), 128);
    }

    #[test]
    fn test_get_set_very_unsafely() {
        let mut state = State::new(101);
        for i in 2..100 {
            unsafe {
                assert_eq!(state.get_state_very_unsafely(i), false);
                assert_eq!(state.get_updated_very_unsafely(i), false);

                state.set_very_unsafely(i, true);

                assert_eq!(state.get_state_very_unsafely(i), true);
                assert_eq!(state.get_updated_very_unsafely(i), true);

                state.set_very_unsafely(i, false);

                assert_eq!(state.get_state_very_unsafely(i), false);
                assert_eq!(state.get_updated_very_unsafely(i), true);
            }
        }
    }

    #[test]
    fn test_get_if_updated_very_unsafely() {
        let mut state = State::new(2);

        unsafe {
            assert_eq!(state.get_if_updated_very_unsafely(0), None);

            state.set(0, true);
            assert_eq!(state.get_if_updated_very_unsafely(0), Some(true));

            state.tick();
            assert_eq!(state.get_if_updated_very_unsafely(0), None);
        }
    }

    #[test]
    #[should_panic(expected = "Tried to access index:64 >= State::len():64")]
    fn test_get_state_very_unsafely_panics() {
        let state = State::new(1);
        unsafe {
            state.get_state_very_unsafely(64);
        }
    }

    #[test]
    #[should_panic(expected = "Tried to access index:65 >= State::len():64")]
    fn test_get_updated_very_unsafely_panics() {
        let state = State::new(1);
        unsafe {
            state.get_updated_very_unsafely(65);
        }
    }

    #[test]
    #[should_panic(expected = "Tried to access index:65 >= State::len():64")]
    fn test_get_if_updated_very_unsafely_panics() {
        let state = State::new(1);
        unsafe {
            state.get_if_updated_very_unsafely(65);
        }
    }

    #[test]
    #[should_panic(expected = "Tried to write to index:65 >= State::len():64")]
    fn test_set_very_unsafely_panics() {
        let mut state = State::new(1);
        unsafe {
            state.set_very_unsafely(65, true);
        }
    }
}
