use super::GateIndex;
use super::InitializedGateGraph;
use concat_idents::concat_idents;
use smallvec::SmallVec;

/// Data structure that represents a probe into a gate graph, whenever any of the gates in the probe changes it's state,
/// The new value of all of the bits will be printed to stdout along with the name.
#[derive(Debug, Clone)]
#[cfg(feature = "debug_gates")]
pub(super) struct Probe {
    pub name: String,
    pub bits: SmallVec<[GateIndex; 1]>,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
macro_rules! circuit_outputs {
    ($ty:ident,$($rest:ident),*) => {
        circuit_outputs!($ty);
        circuit_outputs!($($rest),*);
    };
    ($ty:ident) => {
        concat_idents!(collect_t = collect, _, $ty, _, lossy {
            pub fn $ty(self, g: &InitializedGateGraph) -> $ty {
                g.collect_t(&g.get_output_handle(self).bits)
            }
        });
        concat_idents!(print_t = print, _, $ty {
            pub fn print_t(self, g: &InitializedGateGraph) {
                println!("{}: {}", &g.get_output_handle(self).name, self.$ty(g));
            }
        });
    };
}
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct OutputHandle(pub(super) usize);
#[derive(Debug, Clone)]
pub struct Output {
    pub(super) name: String,
    pub(super) bits: SmallVec<[GateIndex; 1]>,
}
impl OutputHandle {
    circuit_outputs!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, char);
    pub fn bx(self, g: &InitializedGateGraph, n: usize) -> bool {
        g.value(g.get_output_handle(self).bits[n])
    }
    pub fn b0(&self, g: &InitializedGateGraph) -> bool {
        self.bx(g, 0)
    }
}
