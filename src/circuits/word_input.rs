use smallvec::SmallVec;

use crate::data_structures::BitIter;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("WI:{}", name)
}
pub struct WordInput {
    levers: Vec<LeverHandle>,
}
impl WordInput {
    pub fn new<S: Into<String>>(g: &mut GateGraphBuilder, width: usize, name: S) -> Self {
        let name = mkname(name.into());
        Self {
            levers: (0..width).map(|_| g.lever(name.clone())).collect(),
        }
    }

    pub fn update_bit(&self, g: &mut InitializedGateGraph, bit: usize, value: bool) -> Option<()> {
        let lever = self.levers.get(bit)?;
        g.update_lever(*lever, value);
        Some(())
    }
    pub fn flip_bit(&self, g: &mut InitializedGateGraph, bit: usize) -> Option<()> {
        let lever = self.levers.get(bit)?;
        g.flip_lever(*lever);
        Some(())
    }
    pub fn set_bit(&self, g: &mut InitializedGateGraph, bit: usize) -> Option<()> {
        self.update_bit(g, bit, true)
    }
    pub fn reset_bit(&self, g: &mut InitializedGateGraph, bit: usize) -> Option<()> {
        self.update_bit(g, bit, true)
    }
    pub fn set<T: Copy + Sized + 'static>(&self, g: &mut InitializedGateGraph, val: T) {
        g.update_levers(&self.levers, BitIter::new(val));
    }
    pub fn reset(&self, g: &mut InitializedGateGraph) {
        g.update_levers(&self.levers, (0..self.levers.len()).map(|_| false));
    }

    pub fn bits(&self) -> SmallVec<[GateIndex; 8]> {
        self.levers.iter().map(|lever| lever.bit()).collect()
    }
    pub fn len(&self) -> usize {
        self.levers.len()
    }
    pub fn is_empty(&self) -> bool {
        self.levers.len() == 0
    }
}
