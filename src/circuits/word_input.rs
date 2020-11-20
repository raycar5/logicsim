use crate::bititer::BitIter;
use crate::graph::*;

pub const WORD_INPUT: &str = "word_input";
pub struct WordInput {
    levers: Vec<GateIndex>,
}
impl WordInput {
    pub fn new(g: &mut GateGraph, width: usize) -> Self {
        Self {
            levers: (0..width).step_by(1).map(|_| g.lever(WORD_INPUT)).collect(),
        }
    }

    pub fn update_bit(&self, g: &mut GateGraph, bit: usize, value: bool) -> Option<()> {
        let lever = self.levers.get(bit)?;
        g.update_lever(*lever, value);
        Some(())
    }
    pub fn flip_bit(&self, g: &mut GateGraph, bit: usize) -> Option<()> {
        let lever = self.levers.get(bit)?;
        g.flip_lever(*lever);
        Some(())
    }
    pub fn set_bit(&self, g: &mut GateGraph, bit: usize) -> Option<()> {
        self.update_bit(g, bit, true)
    }
    pub fn reset_bit(&self, g: &mut GateGraph, bit: usize) -> Option<()> {
        self.update_bit(g, bit, true)
    }
    pub fn set<T: std::fmt::Debug>(&self, g: &mut GateGraph, val: T) {
        let width = std::mem::size_of_val(&val) * 8;
        assert!(
            width <= self.levers.len(),
            "not enough bits in word input to set value {:?}",
            val
        );
        g.update_levers(&self.levers, BitIter::new(val));
    }

    pub fn bits(&self) -> &[GateIndex] {
        &self.levers
    }
}
