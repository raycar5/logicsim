use smallvec::SmallVec;

use crate::data_structures::BitIter;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("WI:{}", name)
}
/// Data Structure that allows you to easily manage a group of [LeverHandles](LeverHandle).
///
/// # Example
/// ```
/// # use logicsim::{GateGraphBuilder,WordInput};
/// # let mut g = GateGraphBuilder::new();
/// let input = WordInput::new(&mut g, 3, "input");
///
/// let output = g.output(&input.bits(), "result");
///
/// let ig = &mut g.init();
///
/// assert_eq!(output.u8(ig), 0);
///
/// input.set_to(ig, 2);
/// assert_eq!(output.u8(ig), 2);
///
/// input.set_bit(ig, 0);
/// assert_eq!(output.u8(ig), 3);
///
/// input.flip_bit(ig, 1);
/// assert_eq!(output.u8(ig), 1);
/// ```
pub struct WordInput {
    levers: Vec<LeverHandle>,
}
// TODO "_stable" versions.
impl WordInput {
    /// Returns a new [WordInput] of width `width` with name `name`.
    pub fn new<S: Into<String>>(g: &mut GateGraphBuilder, width: usize, name: S) -> Self {
        let name = mkname(name.into());
        Self {
            levers: (0..width).map(|_| g.lever(name.clone())).collect(),
        }
    }

    /// Sets the lever at index `bit` to `value`.
    pub fn update_bit(&self, g: &mut InitializedGateGraph, bit: usize, value: bool) -> Option<()> {
        let lever = self.levers.get(bit)?;
        g.update_lever(*lever, value);
        Some(())
    }

    /// Flips the lever at index `bit`.
    pub fn flip_bit(&self, g: &mut InitializedGateGraph, bit: usize) -> Option<()> {
        let lever = self.levers.get(bit)?;
        g.flip_lever(*lever);
        Some(())
    }

    /// Sets the lever at index `bit` to true.
    pub fn set_bit(&self, g: &mut InitializedGateGraph, bit: usize) -> Option<()> {
        self.update_bit(g, bit, true)
    }

    /// Sets the lever at index `bit` to false.
    pub fn reset_bit(&self, g: &mut InitializedGateGraph, bit: usize) -> Option<()> {
        self.update_bit(g, bit, true)
    }

    /// Sets the levers to the native endian bits of `value`.
    /// If [size_of_val](std::mem::size_of_val)(value) > self.len(), it will ignore the excess bits.
    /// If [size_of_val](std::mem::size_of_val)(value) < self.len(), it will 0 extend the value.
    pub fn set_to<T: Copy + Sized + 'static>(&self, g: &mut InitializedGateGraph, value: T) {
        g.update_levers(&self.levers, BitIter::new(value));
    }

    /// Sets all the levers to true.
    pub fn set(&self, g: &mut InitializedGateGraph) {
        g.update_levers(&self.levers, (0..self.levers.len()).map(|_| false));
    }

    /// Sets all the levers to false.
    pub fn reset(&self, g: &mut InitializedGateGraph) {
        g.update_levers(&self.levers, (0..self.levers.len()).map(|_| false));
    }

    /// Returns a [SmallVec]<[GateIndex]> to connect to other components.
    pub fn bits(&self) -> SmallVec<[GateIndex; 8]> {
        self.levers.iter().map(|lever| lever.bit()).collect()
    }

    /// Returns the width of the [WordInput].
    pub fn len(&self) -> usize {
        self.levers.len()
    }

    /// Returns true the width of the [WordInput] == 0.
    pub fn is_empty(&self) -> bool {
        self.levers.len() == 0
    }
}
