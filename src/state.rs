use crate::bititer::word_mask_64;
use crate::graph::GateIndex;
use pretty_hex::*;
#[derive(Hash)]
pub struct State {
    states: Vec<u64>,
    updated: Vec<u64>,
}
impl State {
    pub fn new() -> State {
        let states = Vec::new();
        let updated = Vec::new();

        State { states, updated }
    }
    pub fn reserve(&mut self, n: usize) {
        self.states.reserve(n / 64);
        self.updated.reserve(n / 64);
    }
    #[inline(always)]
    fn get_from_bit_vec(v: &Vec<u64>, real_index: usize) -> bool {
        let (word_index, mask) = word_mask_64(real_index);
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

        Self::get_from_bit_vec(&self.states, index.idx)
    }

    pub fn get_updated(&self, index: GateIndex) -> bool {
        if index.is_off() || index.is_on() {
            return true;
        }

        Self::get_from_bit_vec(&self.updated, index.idx)
    }

    pub fn get_if_updated(&self, index: GateIndex) -> Option<bool> {
        if self.get_updated(index) {
            Some(self.get_state(index))
        } else {
            None
        }
    }

    fn reserve_for_word(&mut self, word_index: usize) {
        let len = self.states.len();
        let diff = word_index as i64 + 1 - len as i64;
        if diff > 0 {
            self.states.reserve(diff as usize);
            self.updated.reserve(diff as usize);

            self.states.extend((0..diff).map(|_| 0));
            self.updated.extend((0..diff).map(|_| 0));
        }
    }

    pub fn set(&mut self, index: GateIndex, value: bool) {
        let (word_index, mask) = word_mask_64(index.idx);

        self.reserve_for_word(word_index);

        let state = &mut self.states[word_index];
        if value {
            *state = *state | mask;
        } else {
            *state = *state & !mask;
        }

        let updated = &mut self.updated[word_index];
        *updated = *updated | mask;
    }

    pub fn set_updated(&mut self, index: GateIndex) {
        let (word_index, mask) = word_mask_64(index.idx);
        self.reserve_for_word(word_index);
        let updated = &mut self.updated[word_index];
        *updated = *updated | mask;
    }

    pub fn tick(&mut self) {
        for updated in &mut self.updated {
            *updated = 0
        }
    }

    pub fn dump(&self) {
        let slice = unsafe {
            std::slice::from_raw_parts(
                self.states.as_ptr() as *const u8,
                self.states.len() * std::mem::size_of::<u64>(),
            )
        };
        println!("{}", pretty_hex(&slice));
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
