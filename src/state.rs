use crate::graph::{OFF, ON};
use pretty_hex::*;
#[derive(Hash)]
pub struct State {
    states: Vec<u64>,
    updated: Vec<u64>,
}
fn word_mask(index: usize) -> (usize, u64) {
    let word = index / 64;
    let mask = 1 << (index % 64);
    (word, mask)
}
impl State {
    pub fn new(size: usize) -> State {
        let len = size / 64;
        let mut states = Vec::new();
        let mut updated = Vec::new();
        states.reserve(len);
        updated.reserve(len);
        State { states, updated }
    }
    #[inline(always)]
    fn get_from_bit_vec(vec: &Vec<u64>, index: usize) -> bool {
        let (word_index, mask) = word_mask(index);
        let word = vec.get(word_index);
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
        Self::get_from_bit_vec(&self.states, index)
    }
    pub fn get_updated(&self, index: usize) -> bool {
        if index == OFF || index == ON {
            return true;
        }
        Self::get_from_bit_vec(&self.updated, index)
    }
    pub fn set(&mut self, index: usize, value: bool) {
        let (word_index, mask) = word_mask(index);

        let len = self.states.len();
        let diff = word_index as i64 + 1 - len as i64;
        if diff > 0 {
            self.states.reserve(diff as usize);
            self.updated.reserve(diff as usize);
            self.states.extend((0..diff).step_by(1).map(|_| 0u64));
            self.updated.extend((0..diff).step_by(1).map(|_| 0u64));
        }

        let updated = &mut self.updated[word_index];
        *updated = *updated | mask;

        let state = &mut self.states[word_index];
        if value {
            *state = *state | mask;
        } else {
            *state = *state & !mask;
        }
    }
    pub fn flip(&mut self, index: usize) {
        let (word_index, mask) = word_mask(index);
        let updated = &mut self.updated[word_index];
        *updated = *updated ^ mask;
    }
    pub fn tick(&mut self) {
        self.updated.iter_mut().for_each(|x| *x = 0);
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
