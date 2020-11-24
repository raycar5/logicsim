use super::InitializedGateGraph;
use indexmap::IndexSet;
use smallvec::SmallVec;
use std::fmt::{self, Display, Formatter};

#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct GateIndex {
    pub idx: usize,
}
impl Display for GateIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.idx)
    }
}
#[macro_export]
macro_rules! gi {
    ( $x:expr ) => {{
        GateIndex::new($x)
    }};
}
pub const OFF: GateIndex = gi!(0);
pub const ON: GateIndex = gi!(1);

impl GateIndex {
    pub const fn new(idx: usize) -> GateIndex {
        GateIndex { idx }
    }
    pub fn is_off(&self) -> bool {
        *self == OFF
    }
    pub fn is_on(&self) -> bool {
        *self == ON
    }
    #[inline(always)]
    pub fn is_const(&self) -> bool {
        *self == OFF || *self == ON
    }
    pub fn opposite_if_const(&self) -> Option<GateIndex> {
        if self.is_on() {
            Some(OFF)
        } else if self.is_off() {
            Some(ON)
        } else {
            None
        }
    }
}
#[derive(Clone, Debug)]
pub(super) enum GateType {
    Off,
    On,
    Lever,
    Xor,
    Xnor,
    Not,
    Or,
    And,
    Nand,
    Nor,
}
use GateType::*;
impl GateType {
    #[inline(always)]
    pub fn accumulate(&self, acc: bool, b: bool) -> bool {
        match self {
            Or | Nor => acc | b,
            And | Nand => acc & b,
            Xor | Xnor => acc ^ b,
            On | Off | Lever | Not => unreachable!(),
        }
    }
    #[inline(always)]
    pub fn init(&self) -> bool {
        match self {
            Or | Nor | Xor | Xnor => false,
            And | Nand => true,
            Not => false,
            On | Off | Lever => unreachable!(),
        }
    }
    #[inline(always)]
    pub fn short_circuits(&self) -> bool {
        match self {
            Xor | Xnor => false,
            Or | Nor | And | Nand => true,
            Not | On | Off | Lever => unreachable!(),
        }
    }
    pub fn is_lever(&self) -> bool {
        matches!(self, Lever)
    }
    pub fn is_negated(&self) -> bool {
        matches!(self, Nor | Nand | Not | Xnor)
    }
}
impl Display for GateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Lever => write!(f, "Lever"),
            On => write!(f, "On"),
            Off => write!(f, "Off"),
            Not => write!(f, "Not"),
            Or => write!(f, "Or"),
            Nor => write!(f, "Nor"),
            And => write!(f, "And"),
            Nand => write!(f, "Nand"),
            Xor => write!(f, "Xor"),
            Xnor => write!(f, "Xnor"),
        }
    }
}

pub(super) const GATE_TINYVEC_SIZE: usize = 2;

#[derive(Debug, Clone)]
pub(super) struct Gate<T> {
    pub ty: GateType,
    pub dependencies: SmallVec<[GateIndex; GATE_TINYVEC_SIZE]>,
    pub dependents: T,
}
impl<T: Default> Gate<T> {
    pub fn new(ty: GateType, dependencies: SmallVec<[GateIndex; GATE_TINYVEC_SIZE]>) -> Self {
        Gate {
            ty,
            dependencies,
            dependents: Default::default(),
        }
    }
}
pub(super) type BuildGate = Gate<IndexSet<GateIndex>>;
pub(super) type InitializedGate = Gate<SmallVec<[GateIndex; 2]>>;
impl From<BuildGate> for InitializedGate {
    fn from(g: BuildGate) -> Self {
        let BuildGate {
            ty,
            dependents,
            dependencies,
        } = g;
        Self {
            ty,
            dependencies,
            dependents: dependents.into_iter().collect(),
        }
    }
}

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
