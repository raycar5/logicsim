use crate::graph::{OFF, ON};
use pretty_hex::*;
#[derive(Hash)]
pub struct State {
    states: Vec<u64>,
}
fn word_mask(index: usize) -> (usize, u64) {
    let word = index / 64;
    let mask = 1 << (index % 64);
    (word, mask)
}
impl State {
    pub fn new(size: usize) -> State {
        let len = size * 2 / 64;
        let mut states = Vec::new();
        states.reserve(len);
        State { states }
    }
    #[inline(always)]
    fn get_from_bit_vec(&self, real_index: usize) -> bool {
        let (word_index, mask) = word_mask(real_index);
        let word = self.states.get(word_index);
        if let Some(word) = word {
            word & mask != 0
        } else {
            false
        }
    }
    pub fn get_state(&self, index: usize) -> bool {
        if index == OFF {
            return false;
        }
        if index == ON {
            return true;
        }
        self.get_from_bit_vec(index * 2)
    }
    pub fn get_updated(&self, index: usize) -> bool {
        if index == OFF || index == ON {
            return true;
        }
        self.get_from_bit_vec(index * 2 + 1)
    }
    pub fn set(&mut self, index: usize, value: bool) {
        let (word_index, mask) = word_mask(index * 2);
        let updated_bit_mask = mask << 1;

        let len = self.states.len();
        let diff = word_index as i64 + 1 - len as i64;
        if diff > 0 {
            self.states.reserve(diff as usize);
            self.states.extend((0..diff).step_by(1).map(|_| 0u64));
        }

        let state = &mut self.states[word_index];
        *state = *state | updated_bit_mask;
        if value {
            *state = *state | mask;
        } else {
            *state = *state & !mask;
        }
    }
    pub fn flip(&mut self, index: usize) {
        let (word_index, mask) = word_mask(index * 2);
        let state = &mut self.states[word_index];
        *state = *state ^ mask;
    }
    pub fn tick(&mut self) {
        // clear all odd bits;
        for state in &mut self.states {
            let mask: u64 = 0x5555555555555555; // pattern 010101 etc..
            *state = *state & mask;
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
        for i in 0..100 {
            let mut state = State::new(1);
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
        let mut state = State::new(1);
        for i in 0..100 {
            assert_eq!(state.get_state(i), false, "index: {}", i);
            assert_eq!(state.get_updated(i), false, "index: {}", i);

            state.set(i, true);

            assert_eq!(state.get_state(i), true, "index: {}", i);
            assert_eq!(state.get_updated(i), true, "index: {}", i);

            state.dump();
            state.tick();
            state.dump();

            assert_eq!(state.get_state(i), true, "index: {}", i);
            assert_eq!(state.get_updated(i), false, "index: {}", i);
        }
    }
}
