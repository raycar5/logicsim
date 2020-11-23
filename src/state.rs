use crate::bititer::word_mask_64;
use crate::graph::GateIndex;
use num_integer::div_ceil;
use pretty_hex::*;
#[derive(Hash)]
pub struct State {
    data: Vec<u64>,
}
enum BitType {
    BitState,
    Updated,
}
use BitType::*;
impl State {
    pub fn new() -> State {
        State {
            data: Default::default(),
        }
    }
    pub fn fill_zero(&mut self, n: usize) {
        self.data = vec![0; div_ceil(n, 64) * 2];
    }

    #[inline(always)]
    fn word_mask(index: usize, ty: BitType) -> (usize, u64) {
        let (word_index, mask) = word_mask_64(index);
        if matches!(ty, BitState) {
            (word_index * 2, mask)
        } else {
            (word_index * 2 + 1, mask)
        }
    }

    #[inline(always)]
    fn get_from_bit_vec(v: &[u64], index: usize, ty: BitType) -> bool {
        let (word_index, mask) = Self::word_mask(index, ty);
        let word = v.get(word_index);
        if let Some(word) = word {
            word & mask != 0
        } else {
            false
        }
    }

    pub fn get_state(&self, index: GateIndex) -> bool {
        if index.is_off() {
            return false;
        }
        if index.is_on() {
            return true;
        }

        Self::get_from_bit_vec(&self.data, index.idx, BitState)
    }

    pub fn get_updated(&self, index: GateIndex) -> bool {
        if index.is_off() || index.is_on() {
            return true;
        }

        Self::get_from_bit_vec(&self.data, index.idx, Updated)
    }

    pub fn get_if_updated(&self, index: GateIndex) -> Option<bool> {
        if self.get_updated(index) {
            Some(self.get_state(index))
        } else {
            None
        }
    }

    fn reserve_for_word(&mut self, word_index: usize) {
        let len = self.data.len();
        let diff = word_index as i64 + 1 - len as i64;
        if diff > 0 {
            self.data.reserve(diff as usize);

            self.data.extend((0..diff).map(|_| 0));
        }
    }

    pub fn set(&mut self, index: GateIndex, value: bool) {
        let (state_word_index, mask) = Self::word_mask(index.idx, BitState);
        let update_word_index = state_word_index + 1;
        self.reserve_for_word(update_word_index);

        let state = &mut self.data[state_word_index];
        if value {
            *state |= mask;
        } else {
            *state &= !mask;
        }

        let update = &mut self.data[update_word_index];
        *update |= mask;
    }

    pub fn set_updated(&mut self, index: GateIndex) {
        let (word_index, mask) = Self::word_mask(index.idx, Updated);

        self.reserve_for_word(word_index);

        let word = &mut self.data[word_index];
        *word |= mask;
    }

    pub fn tick(&mut self) {
        for updated in self.data.iter_mut().skip(1).step_by(2) {
            *updated = 0
        }
    }

    pub fn dump(&self) {
        // This is safe because a slice of u64 can be safely reinterpreted into
        // a slice of u8.
        let slice = unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const u8,
                self.data.len() * std::mem::size_of::<u64>(),
            )
        };
        println!("{}", pretty_hex(&slice));
    }

    // The dark corner.
    /// # Safety
    /// This function is safe if real_index < v.len() .
    /// This invariant is checked in debug mode.
    #[inline(always)]
    unsafe fn get_from_bit_vec_very_unsafely(v: &[u64], index: usize, ty: BitType) -> bool {
        let (word_index, mask) = Self::word_mask(index, ty);
        debug_assert!(word_index < v.len());

        let word = v.get_unchecked(word_index);
        word & mask != 0
    }
    /// # Safety
    /// This function is safe if index < [State::len()].
    /// This invariant is checked in debug mode.
    pub unsafe fn get_state_very_unsafely(&self, index: GateIndex) -> bool {
        Self::get_from_bit_vec_very_unsafely(&self.data, index.idx, BitState)
    }
    /// # Safety
    /// This function is safe if index < [State::len()].
    /// This invariant is checked in debug mode.
    pub unsafe fn get_updated_very_unsafely(&self, index: GateIndex) -> bool {
        Self::get_from_bit_vec_very_unsafely(&self.data, index.idx, Updated)
    }
    /// # Safety
    /// This function is safe if index < [State::len()].
    /// This invariant is checked in debug mode.
    pub unsafe fn get_if_updated_very_unsafely(&self, index: GateIndex) -> Option<bool> {
        if self.get_updated_very_unsafely(index) {
            Some(self.get_state_very_unsafely(index))
        } else {
            None
        }
    }
    /// # Safety
    /// This function is safe if index < [State::len()].
    /// This invariant is checked in debug mode.
    pub unsafe fn set_very_unsafely(&mut self, index: GateIndex, value: bool) {
        let (state_word_index, mask) = Self::word_mask(index.idx, BitState);
        let updated_word_index = state_word_index + 1;
        debug_assert!(updated_word_index < self.data.len());

        let state = self.data.get_unchecked_mut(state_word_index);
        if value {
            *state |= mask;
        } else {
            *state &= !mask;
        }
        let updated = self.data.get_unchecked_mut(updated_word_index);
        *updated |= mask;
    }
}
impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_set() {
        for i in 2..100 {
            let mut state = State::new();
            assert_eq!(state.get_state(gi!(i)), false);
            assert_eq!(state.get_updated(gi!(i)), false);

            state.set(gi!(i), true);

            assert_eq!(state.get_state(gi!(i)), true);
            assert_eq!(state.get_updated(gi!(i)), true);

            state.set(gi!(i), false);

            assert_eq!(state.get_state(gi!(i)), false);
            assert_eq!(state.get_updated(gi!(i)), true);
        }
    }

    #[test]
    fn test_tick() {
        let mut state = State::new();
        for i in 2..100 {
            assert_eq!(state.get_state(gi!(i)), false, "index: {}", i);
            assert_eq!(state.get_updated(gi!(i)), false, "index: {}", i);

            state.set(gi!(i), true);

            assert_eq!(state.get_state(gi!(i)), true, "index: {}", i);
            assert_eq!(state.get_updated(gi!(i)), true, "index: {}", i);

            state.tick();
            assert_eq!(state.get_state(gi!(i)), true, "index: {}", i);
            assert_eq!(state.get_updated(gi!(i)), false, "index: {}", i);
        }
    }
}
