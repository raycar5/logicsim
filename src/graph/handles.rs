use super::GateIndex;
use super::InitializedGateGraph;
use smallvec::SmallVec;

#[derive(Debug, Clone)]
#[cfg(feature = "debug_gates")]
pub(super) struct Probe {
    pub name: String,
    pub bits: SmallVec<[GateIndex; 1]>,
}
#[derive(Debug, Copy, Clone)]
pub struct LeverHandle {
    pub(super) handle: usize,
    pub(super) idx: GateIndex,
}
impl LeverHandle {
    // This should be fine since you can't do much with the GateIndex
    // once the graph is initialized.
    pub fn bit(&self) -> GateIndex {
        self.idx
    }
}
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct CircuitOutputHandle(pub(super) usize);
// TODO macro this?
#[derive(Debug, Clone)]
pub struct CircuitOutput {
    pub(super) name: String,
    pub(super) bits: SmallVec<[GateIndex; 1]>,
}
impl CircuitOutputHandle {
    pub fn u8(self, g: &InitializedGateGraph) -> u8 {
        g.collect_u8_lossy(&g.get_output_handle(self).bits)
    }
    pub fn i8(&self, g: &InitializedGateGraph) -> i8 {
        self.u8(g) as i8
    }
    pub fn u128(self, g: &InitializedGateGraph) -> u128 {
        g.collect_u128_lossy(&g.get_output_handle(self).bits)
    }
    pub fn i128(&self, g: &InitializedGateGraph) -> i128 {
        self.u128(g) as i128
    }
    pub fn char(&self, g: &InitializedGateGraph) -> char {
        self.u8(g) as char
    }
    pub fn print_u8(self, g: &InitializedGateGraph) {
        println!("{}: {}", &g.get_output_handle(self).name, self.u8(g));
    }
    pub fn print_i8(self, g: &InitializedGateGraph) {
        println!("{}: {}", &g.get_output_handle(self).name, self.i8(g));
    }
    pub fn bx(self, g: &InitializedGateGraph, n: usize) -> bool {
        g.value(g.get_output_handle(self).bits[n])
    }
    pub fn b0(&self, g: &InitializedGateGraph) -> bool {
        self.bx(g, 0)
    }
}
