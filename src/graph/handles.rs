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
/// Handle type that represents a lever gate in an [InitializedGateGraph] or [GateGraphBuilder](super::GateGraphBuilder)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LeverHandle {
    pub(super) handle: usize,
    pub(super) idx: GateIndex,
}
impl LeverHandle {
    /// Returns the [GateIndex] of the lever gate.
    pub fn bit(&self) -> GateIndex {
        self.idx
    }
}

/// Generates the type() functions for [Output].
macro_rules! circuit_outputs {
    ($ty:ident,$($rest:ident),*) => {
        circuit_outputs!($ty);
        circuit_outputs!($($rest),*);
    };
    ($ty:ident) => {
        concat_idents!(collect_t = collect, _, $ty, _, lossy {
            /// Returns a value of the corresponding type created from
            /// the current state bits in the output.
            ///
            /// If there are more bits than [size_of::\<type\>](std::mem::size_of),
            /// the excess bits will be ignored.
            ///
            /// If there are less bits, the value will be 0 extended.
            pub fn $ty(self, g: &InitializedGateGraph) -> $ty {
                g.collect_t(&g.get_output(self).bits)
            }
        });
        concat_idents!(print_t = print, _, $ty {
            /// Prints the output of the corresponding type() function along with
            /// the name of the output.
            pub fn print_t(self, g: &InitializedGateGraph) {
                println!("{}: {}", &g.get_output(self).name, self.$ty(g));
            }
        });
    };
}

/// Handle type that represents a set of gates in an [InitializedGateGraph]
/// or [GateGraphBuilder](super::GateGraphBuilder) which we want to query.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct OutputHandle(pub(super) usize);

/// Data structure that stores a set of gates in an [InitializedGateGraph]
/// or [GateGraphBuilder](super::GateGraphBuilder) which we want to query. Along with a name.
#[derive(Debug, Clone)]
pub(super) struct Output {
    pub(super) name: String,
    pub(super) bits: SmallVec<[GateIndex; 1]>,
}
impl OutputHandle {
    circuit_outputs!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, char);

    // Returns the state of the `n` bit of the output.
    pub fn bx(self, g: &InitializedGateGraph, n: usize) -> bool {
        g.value(g.get_output(self).bits[n])
    }

    // Returns the state of the 0th bit of the output.
    pub fn b0(&self, g: &InitializedGateGraph) -> bool {
        self.bx(g, 0)
    }
}
