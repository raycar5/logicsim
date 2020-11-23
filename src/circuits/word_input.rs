use crate::bititer::BitIter;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("WI:{}", name)
}
pub struct WordInput {
    levers: Vec<GateIndex>,
}
impl WordInput {
    pub fn new<S: Into<String>>(g: &mut GateGraph, width: usize, name: S) -> Self {
        let name = mkname(name.into());
        Self {
            levers: (0..width).map(|_| g.lever(name.clone())).collect(),
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
    pub fn set<T: Copy>(&self, g: &mut GateGraph, val: T) {
        g.update_levers(&self.levers, BitIter::new(val));
    }

    pub fn bits(&self) -> &[GateIndex] {
        &self.levers
    }
    pub fn len(&self) -> usize {
        self.levers.len()
    }
    pub fn is_empty(&self) -> bool {
        self.levers.len() == 0
    }
}
