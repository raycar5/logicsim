use crate::data_structures::SlabIndex;

use indexmap::IndexSet;
use smallvec::SmallVec;
use std::fmt::{self, Display, Formatter};

/// Represents the index of a logic gate in a [super::GateGraphBuilder].
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct GateIndex {
    pub(super) idx: usize,
}

/// Returns a new GateIndex from a provided usize.
macro_rules! gi {
    ( $x:expr ) => {{
        GateIndex::new($x)
    }};
}

/// The [GateIndex] of the OFF constant in any [GateGraphBuilder](super::GateGraphBuilder).
///
/// Having it be a constant greatly simplifies both implementation and use.
pub const OFF: GateIndex = gi!(0);
/// The [GateIndex] of the ON constant in any [GateGraphBuilder](super::GateGraphBuilder).
///
/// Having it be a constant greatly simplifies both implementation and use.
pub const ON: GateIndex = gi!(1);

impl GateIndex {
    /// Returns a new GateIndex from a provided usize.
    pub(super) const fn new(idx: usize) -> GateIndex {
        GateIndex { idx }
    }

    /// Returns true if `self` is the index of the OFF constant.
    pub fn is_off(&self) -> bool {
        *self == OFF
    }

    /// Returns true if `self` is the index of the ON constant.
    pub fn is_on(&self) -> bool {
        *self == ON
    }

    /// Returns true if `self` is [ON] or [OFF].
    ///
    /// # Example
    /// ```
    /// # use wires::{GateGraphBuilder,ON,OFF};
    /// let mut g = GateGraphBuilder::new();
    ///
    /// let and = g.and("and");
    /// assert_eq!(and.is_const(), false);
    ///
    ///
    /// assert_eq!(ON.is_const(), true);
    /// assert_eq!(OFF.is_const(), true);
    /// ```
    #[inline(always)]
    pub fn is_const(&self) -> bool {
        *self == OFF || *self == ON
    }

    /// Returns Some(OFF) if `self` is ON, Some(ON) if `self` is off, None otherwise.
    /// # Example
    /// ```
    /// # use wires::{GateGraphBuilder,ON,OFF};
    /// let mut g = GateGraphBuilder::new();
    ///
    /// let and = g.and("and");
    /// assert_eq!(and.opposite_if_const(), None);
    ///
    ///
    /// assert_eq!(ON.opposite_if_const(), Some(OFF));
    /// assert_eq!(OFF.opposite_if_const(), Some(ON));
    /// ```
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

impl From<SlabIndex> for GateIndex {
    fn from(i: SlabIndex) -> Self {
        Self {
            idx: i.i_actually_really_know_what_i_am_doing_and_i_want_the_inner_usize(),
        }
    }
}
impl Into<SlabIndex> for GateIndex {
    fn into(self) -> SlabIndex {
        SlabIndex::i_actually_really_know_what_i_am_doing_and_i_want_to_construct_from_usize(
            self.idx,
        )
    }
}
impl Into<SlabIndex> for &GateIndex {
    fn into(self) -> SlabIndex {
        SlabIndex::i_actually_really_know_what_i_am_doing_and_i_want_to_construct_from_usize(
            self.idx,
        )
    }
}
impl Display for GateIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.idx)
    }
}

/// Enum representing the different types of gates in a gate graph.
#[repr(u8)]
#[derive(Clone, Debug, Copy, Eq, PartialEq, Hash)]
pub(super) enum GateType {
    Off = 0,
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
    /// Calculates the new state of a gate from the state of it's dependencies.
    /// Keep in mind if the gate [is negated](GateType::is_negated) the result should be negated.
    ///
    /// # Example
    // RustDoc doesn't like doctests for private fields...
    /// ```compile_fail
    /// assert_eq!(GateType::Or.accumulate(true,false), true);
    /// assert_eq!(GateType::Nor.accumulate(true,false), true);
    ///
    /// assert_eq!(GateType::And.accumulate(true,false), false);
    /// assert_eq!(GateType::NAnd.accumulate(true,false), false);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `self` is On, Off, Lever or Not because those gate types don't have
    /// multiple dependencies.
    #[inline(always)]
    pub fn accumulate(&self, acc: bool, b: bool) -> bool {
        match self {
            Or | Nor => acc | b,
            And | Nand => acc & b,
            Xor | Xnor => acc ^ b,
            On | Off | Lever | Not => {
                unreachable!("Accumulate only works on gates with multiple dependencies")
            }
        }
    }

    /// Returns the corresponding value to initialize the [accumulation](GateType::accumulate) of the new state for
    /// the given [GateType].
    /// In other words, returns the value that will not short circuit or affect the result.
    ///
    /// # Panics
    ///
    /// Panics if `self` is On, Off or Lever because those gate types don't have dependencies.
    #[inline(always)]
    pub fn init(&self) -> bool {
        match self {
            Or | Nor | Xor | Xnor => false,
            And | Nand => true,
            Not => false,
            On | Off | Lever => unreachable!("Init doesn't work on gates without dependencies"),
        }
    }

    /// Returns true if the gate can ignore the rest of the dependencies once a single one has a particular state.
    ///
    /// For example in or gates if a single dependency is on, the gate is on, the state of the rest of the dependencies doesn't matter.
    /// The opposite is true for and gates.
    ///
    /// Xor and Xnor gates on the other hand don't short-circuit therefore we need to know the state of all of it's dependencies to know
    /// the new state.
    ///
    /// # Panics
    ///
    /// Panics if `self` is On, Off, Lever or Not because those gate types don't have
    /// multiple dependencies.
    #[inline(always)]
    pub fn short_circuits(&self) -> bool {
        match self {
            Xor | Xnor => false,
            Or | Nor | And | Nand => true,
            Not | On | Off | Lever => {
                unreachable!("Short_circuits only works on gates with multiple dependencies")
            }
        }
    }

    /// Returns the negated version of a [GateType] if it has one.
    ///
    /// For example Or => Nor, Nand => And etc...
    ///
    /// # Panics
    ///
    /// Panics if `self` is On, Off, Lever or Not because those gate types don't have
    /// a negated equivalent.
    #[inline(always)]
    pub fn negated_version(&self) -> GateType {
        match self {
            Or => Nor,
            Nor => Or,
            And => Nand,
            Nand => And,
            Xor => Xnor,
            Xnor => Xor,
            On | Off | Not | Lever => unreachable!(),
        }
    }

    /// Returns true if the [GateType] has a negated equivalent.
    #[inline(always)]
    pub fn has_negated_version(&self) -> bool {
        !matches!(self, On | Off | Not | Lever)
    }

    /// Returns true if `self` is [Lever].
    pub fn is_lever(&self) -> bool {
        matches!(self, Lever)
    }

    /// Returns true if `self` is [Not].
    pub fn is_not(&self) -> bool {
        matches!(self, Not)
    }

    /// Returns true if `self` is [Not], [Nor], [Nand] or [Xnor].
    pub fn is_negated(&self) -> bool {
        matches!(self, Nor | Nand | Not | Xnor)
    }
}
impl Display for GateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Lever => write!(f, stringify!(Lever)),
            On => write!(f, stringify!(On)),
            Off => write!(f, stringify!(Off)),
            Not => write!(f, stringify!(Not)),
            Or => write!(f, stringify!(Or)),
            Nor => write!(f, stringify!(Nor)),
            And => write!(f, stringify!(And)),
            Nand => write!(f, stringify!(Nand)),
            Xor => write!(f, stringify!(Xor)),
            Xnor => write!(f, stringify!(Xnor)),
        }
    }
}

/// Amount of dependencies kept in the stack for a gate.
/// If a gate has more than GATE_DEPENDENCIES_TINYVEC_SIZE, they will spill into the heap.
pub(super) const GATE_DEPENDENCIES_TINYVEC_SIZE: usize = 2;

/// Data structure which represents a gate node with edges to it's dependencies and dependents.
/// [Gate] is generic over the type of dependent container to provide more optimized containers for
/// build time vs runtime.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(super) struct Gate<T> {
    pub ty: GateType,
    pub dependencies: SmallVec<[GateIndex; GATE_DEPENDENCIES_TINYVEC_SIZE]>,
    pub dependents: T,
}

impl<T: Default> Gate<T> {
    /// Returns a new [Gate] with the given `ty` and `dependencies` and no dependents.
    pub fn new(
        ty: GateType,
        dependencies: SmallVec<[GateIndex; GATE_DEPENDENCIES_TINYVEC_SIZE]>,
    ) -> Self {
        Gate {
            ty,
            dependencies,
            dependents: Default::default(),
        }
    }
}

/// Gate type optimized for build time, the dependents are kept in an ordered set which has good
/// search and iteration characteristics at the expense of size.
pub(super) type BuildGate = Gate<IndexSet<GateIndex>>;

/// Gate type optimized for runtime, the dependents are kept in a [SmallVec] because dependents
/// are not searched at runtime, only iterated.
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

impl BuildGate {
    /// Replaces all occurrences of `old_dep` with `new_dep` in the set of dependency edges.
    pub(super) fn swap_dependency(&mut self, old_dep: GateIndex, new_dep: GateIndex) {
        for d in &mut self.dependencies {
            if old_dep == *d {
                *d = new_dep
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use smallvec::smallvec;

    #[test]
    fn test_swap_dependency() {
        let mut g = Gate::new(Or, smallvec![gi!(3), gi!(2), gi!(3)]);
        g.swap_dependency(gi!(3), gi!(1));

        assert_eq!(g.dependencies[0], gi!(1));
        assert_eq!(g.dependencies[1], gi!(2));
        assert_eq!(g.dependencies[2], gi!(1));
    }
}
