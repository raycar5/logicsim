use crate::bititer::BitIter;
use crate::slab::Slab;
use crate::state::State;
use bitvec::vec::BitVec;
use indexmap::IndexSet;
use smallvec::{smallvec, SmallVec};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{self, Display, Formatter};
use std::path::Path;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct GateIndex {
    pub idx: usize,
}
impl Display for GateIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.idx)
    }
}
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
pub enum GateType {
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
    Lut(BitVec),
}
impl GateType {
    #[inline(always)]
    fn accumulate(&self, acc: bool, b: bool) -> bool {
        match self {
            Or | Nor => acc | b,
            And | Nand => acc & b,
            Xor | Xnor => acc ^ b,
            On | Off | Lever | Not | Lut(..) => unreachable!(),
        }
    }
    #[inline(always)]
    fn init(&self) -> bool {
        match self {
            Or | Nor | Xor | Xnor => false,
            And | Nand => true,
            Not => false,
            On | Off | Lever | Lut(..) => unreachable!(),
        }
    }
    #[inline(always)]
    fn short_circuits(&self) -> bool {
        match self {
            Xor | Xnor => false,
            Or | Nor | And | Nand => true,
            Not | On | Off | Lever | Lut(..) => unreachable!(),
        }
    }
    fn is_lever(&self) -> bool {
        if let Lever = self {
            true
        } else {
            false
        }
    }
    fn is_negated(&self) -> bool {
        if let Nor | Nand | Not | Xnor = self {
            true
        } else {
            false
        }
    }
    fn is_not(&self) -> bool {
        if let Not = self {
            true
        } else {
            false
        }
    }
}
impl Display for GateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Lut(lut) => write!(f, "{}", lut),
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
use GateType::*;

const GATE_TINYVEC_SIZE: usize = 2;
#[derive(Clone)]
struct Gate {
    ty: GateType,
    dependencies: SmallVec<[GateIndex; GATE_TINYVEC_SIZE]>,
    dependents: IndexSet<usize>,
}
impl Gate {
    fn new(ty: GateType, dependencies: SmallVec<[GateIndex; GATE_TINYVEC_SIZE]>) -> Self {
        Gate {
            ty,
            dependencies,
            dependents: Default::default(),
        }
    }
}
#[cfg(feature = "debug_gate_names")]
struct Probe {
    name: String,
    bits: SmallVec<[GateIndex; 1]>,
}
// TODO macro this?
pub struct CircuitOutput {
    name: String,
    bits: SmallVec<[GateIndex; 1]>,
}
impl CircuitOutput {
    pub fn u8(&self, g: &GateGraph) -> u8 {
        g.collect_u8_lossy(&self.bits)
    }
    pub fn i8(&self, g: &GateGraph) -> i8 {
        self.u8(g) as i8
    }
    pub fn u128(&self, g: &GateGraph) -> u128 {
        g.collect_u128_lossy(&self.bits)
    }
    pub fn i128(&self, g: &GateGraph) -> i128 {
        self.u128(g) as i128
    }
    pub fn char(&self, g: &GateGraph) -> char {
        self.u8(g) as char
    }
    pub fn print_u8(&self, g: &GateGraph) {
        println!("{}: {}", self.name, self.u8(g));
    }
    pub fn print_i8(&self, g: &GateGraph) {
        println!("{}: {}", self.name, self.i8(g));
    }
    pub fn bx(&self, g: &GateGraph, n: usize) -> bool {
        g.value(self.bits[n])
    }
    pub fn b0(&self, g: &GateGraph) -> bool {
        self.bx(g, 0)
    }
}

pub struct GateGraph {
    nodes: Slab<Gate>,
    pending_updates: Vec<GateIndex>,
    next_pending_updates: Vec<GateIndex>,
    propagation_queue: VecDeque<GateIndex>, // allocated outside to prevent allocations in the hot loop.
    outputs: HashSet<GateIndex>,
    state: State,
    #[cfg(feature = "debug_gate_names")]
    names: HashMap<GateIndex, String>,
    #[cfg(feature = "debug_gate_names")]
    probes: HashMap<GateIndex, Probe>,
}
impl GateGraph {
    pub fn new() -> GateGraph {
        let mut nodes = Slab::new();
        nodes.insert(Gate {
            ty: Off,
            dependencies: smallvec![],
            dependents: Default::default(),
        });
        nodes.insert(Gate {
            ty: On,
            dependencies: smallvec![],
            dependents: Default::default(),
        });
        GateGraph {
            nodes,
            pending_updates: vec![],
            next_pending_updates: vec![],
            state: State::new(),
            propagation_queue: VecDeque::new(),
            outputs: HashSet::new(),
            #[cfg(feature = "debug_gate_names")]
            names: HashMap::new(),
            #[cfg(feature = "debug_gate_names")]
            probes: HashMap::new(),
        }
    }

    // Dependency operations.
    pub fn dpush(&mut self, idx: GateIndex, new_dep: GateIndex) {
        let gate = self.nodes.get_mut(idx.idx).unwrap();
        match gate.ty {
            Off => assert!(false, "OFF has no dependencies"),
            On => assert!(false, "ON has no dependencies"),
            Lever => assert!(false, "Lever has no dependencies"),
            Or | Nor | And | Nand | Xor | Xnor | Not | Lut(..) => {
                gate.dependencies.push(new_dep);
                self.nodes
                    .get_mut(new_dep.idx)
                    .unwrap()
                    .dependents
                    .insert(idx.idx);
            }
        }
    }
    pub fn dx(&mut self, idx: GateIndex, new_dep: GateIndex, x: usize) {
        let gate = self.nodes.get_mut(idx.idx).unwrap();
        match gate.ty {
            Off => assert!(false, "OFF has no dependencies"),
            On => assert!(false, "ON has no dependencies"),
            Lever => assert!(false, "Lever has no dependencies"),
            Lut(..) => unimplemented!(""),
            Not => {
                assert!(x == 0, "Not only has one dependency");
            }
            // Left explicitly to get errors when a new gate type is added
            Or | Nor | And | Nand | Xor | Xnor => {}
        }

        let old_dep = std::mem::replace(&mut gate.dependencies[x], new_dep);

        self.nodes
            .get_mut(old_dep.idx)
            .unwrap()
            .dependents
            .remove(&idx.idx);
        self.nodes
            .get_mut(new_dep.idx)
            .unwrap()
            .dependents
            .insert(idx.idx);
    }
    pub fn d0(&mut self, gate: GateIndex, dep: GateIndex) {
        self.dx(gate, dep, 0)
    }
    pub fn d1(&mut self, gate: GateIndex, dep: GateIndex) {
        self.dx(gate, dep, 1)
    }

    // Gate operations.
    #[allow(unused_variables)]
    fn create_gate<S: Into<String>>(&mut self, idx: GateIndex, deps: &[GateIndex], name: S) {
        for dep in deps {
            self.nodes
                .get_mut(dep.idx)
                .unwrap()
                .dependents
                .insert(idx.idx);
        }
        #[cfg(feature = "debug_gate_names")]
        self.names.insert(idx, name.into());
    }
    pub fn gate<S: Into<String>>(&mut self, ty: GateType, name: S) -> GateIndex {
        let idx = gi!(self.nodes.insert(Gate::new(ty, smallvec![])));
        self.create_gate(idx, &[], name);
        idx
    }
    pub fn lever<S: Into<String>>(&mut self, name: S) -> GateIndex {
        let idx = GateIndex::new(self.nodes.insert(Gate::new(Lever, smallvec![])));
        self.create_gate(idx, &[], name);
        idx
    }
    pub fn not<S: Into<String>>(&mut self, name: S) -> GateIndex {
        self.not1(OFF, name)
    }
    pub fn not1<S: Into<String>>(&mut self, dep: GateIndex, name: S) -> GateIndex {
        let idx = GateIndex::new(self.nodes.insert(Gate::new(Not, smallvec![dep])));
        self.create_gate(idx, &[dep], name);
        idx
    }
    pub fn or<S: Into<String>>(&mut self, name: S) -> GateIndex {
        let idx = GateIndex::new(self.nodes.insert(Gate::new(Or, smallvec![])));
        self.create_gate(idx, &[], name);
        idx
    }
    pub fn or2<S: Into<String>>(&mut self, d0: GateIndex, d1: GateIndex, name: S) -> GateIndex {
        let idx = GateIndex::new(self.nodes.insert(Gate::new(Or, smallvec![d0, d1])));
        self.create_gate(idx, &[d0, d1], name);
        idx
    }
    pub fn nor<S: Into<String>>(&mut self, name: S) -> GateIndex {
        let idx = GateIndex::new(self.nodes.insert(Gate::new(Nor, smallvec![])));
        self.create_gate(idx, &[], name);
        idx
    }
    pub fn nor1<S: Into<String>>(&mut self, d0: GateIndex, name: S) -> GateIndex {
        let idx = GateIndex::new(self.nodes.insert(Gate::new(Nor, smallvec![d0])));
        self.create_gate(idx, &[d0], name);
        idx
    }
    pub fn nor2<S: Into<String>>(&mut self, d0: GateIndex, d1: GateIndex, name: S) -> GateIndex {
        let idx = GateIndex::new(self.nodes.insert(Gate::new(Nor, smallvec![d0, d1])));
        self.create_gate(idx, &[d0, d1], name);
        idx
    }
    pub fn xor2<S: Into<String>>(&mut self, d0: GateIndex, d1: GateIndex, name: S) -> GateIndex {
        let idx = GateIndex::new(self.nodes.insert(Gate::new(Xor, smallvec![d0, d1])));
        self.create_gate(idx, &[d0, d1], name);
        idx
    }
    pub fn and<S: Into<String>>(&mut self, name: S) -> GateIndex {
        let idx = GateIndex::new(self.nodes.insert(Gate::new(And, smallvec![])));
        self.create_gate(idx, &[], name);
        idx
    }
    pub fn and2<S: Into<String>>(&mut self, d0: GateIndex, d1: GateIndex, name: S) -> GateIndex {
        let idx = GateIndex::new(self.nodes.insert(Gate::new(And, smallvec![d0, d1])));
        self.create_gate(idx, &[d0, d1], name);
        idx
    }
    pub fn nand2<S: Into<String>>(&mut self, d0: GateIndex, d1: GateIndex, name: S) -> GateIndex {
        let idx = GateIndex::new(self.nodes.insert(Gate::new(Nand, smallvec![d0, d1])));
        self.create_gate(idx, &[d0, d1], name);
        idx
    }

    #[inline(always)]
    unsafe fn fold_short(&self, ty: &GateType, gates: &[GateIndex]) -> bool {
        let init = ty.init();
        let short = !init;
        // Using a manual loop results in 2% less instructions.
        for i in 0..gates.len() {
            let state = self.state.get_state_very_unsafely(gates[i]);
            if ty.accumulate(init, state) == short {
                return short;
            }
        }
        init
    }
    // Main VERY HOT loop.
    // The unsafe code was added after careful consideration, profiling and measuring of the performance impact.
    // All unsafe invariants are checked in debug mode using debug_assert!().
    fn tick_inner(&mut self) {
        while let Some(idx) = self.propagation_queue.pop_front() {
            // This is safe because the propagation queue gets filled by items coming from
            // nodes.iter() or levers, both of which are always initialized.
            let node = unsafe { self.nodes.get_very_unsafely(idx.idx) };

            let new_state = match &node.ty {
                On => true,
                Off => false,
                // This is safe because I fill the state on init.
                Lever => unsafe { self.state.get_state_very_unsafely(idx) },
                Not => unsafe { !self.state.get_state_very_unsafely(node.dependencies[0]) },
                Lut(lut) => {
                    let index = self.collect_usize_lossy(&node.dependencies);
                    lut[index]
                }
                Or | Nor | And | Nand | Xor | Xnor => {
                    let mut new_state = if node.dependencies.is_empty() {
                        false
                    } else {
                        if node.ty.short_circuits() {
                            // This is safe because I fill the state on init.
                            unsafe { self.fold_short(&node.ty, &node.dependencies) }
                        } else {
                            let mut result = node.ty.init();
                            // Using a manual loop results in 2% less instructions.
                            for i in 0..node.dependencies.len() {
                                // This is safe because I fill the state on init.
                                let state = unsafe {
                                    self.state.get_state_very_unsafely(node.dependencies[i])
                                };
                                result = node.ty.accumulate(result, state);
                            }
                            result
                        }
                    };
                    if node.ty.is_negated() {
                        new_state = !new_state;
                    }
                    new_state
                }
            };
            // This is safe because I fill the state on init.
            if let Some(old_state) = unsafe { self.state.get_if_updated_very_unsafely(idx) } {
                if old_state != new_state {
                    self.next_pending_updates.push(idx);
                }
                continue;
            }
            // This is safe because I fill the state on init.
            let old_state = unsafe { self.state.get_state_very_unsafely(idx) };
            unsafe { self.state.set_very_unsafely(idx, new_state) };

            #[cfg(feature = "debug_gate_names")]
            if old_state != new_state {
                if let Some(probe) = self.probes.get(&idx) {
                    match probe.bits.len() {
                        0 => {}
                        1 => println!("{}:{}", probe.name, new_state),
                        2..=8 => {
                            println!("{}:{}", probe.name, self.collect_u8_lossy(&probe.bits))
                        }
                        _ => unimplemented!(),
                    }
                }
            }
            if node.ty.is_lever() || old_state != new_state {
                self.propagation_queue
                    .extend(node.dependents.iter().map(|i| gi!(*i)))
            }
        }
    }
    pub fn tick(&mut self) {
        while let Some(pending) = &self.pending_updates.pop() {
            self.state.tick();
            self.propagation_queue.push_back(*pending);
            self.tick_inner()
        }
        self.pending_updates.extend(
            self.next_pending_updates
                .drain(0..self.next_pending_updates.len()),
        )
    }

    pub fn init(&mut self) {
        self.optimize();
        self.init_unoptimized();
    }
    pub fn init_unoptimized(&mut self) {
        self.state.fill_zero(self.nodes.total_len());

        for idx in self.nodes.iter().map(|(i, _)| gi!(i)).collect::<Vec<_>>() {
            if idx != OFF && idx != ON && self.state.get_updated(idx) {
                continue;
            }
            self.propagation_queue.push_back(idx);
            self.tick_inner();
        }
        self.pending_updates.extend(
            self.next_pending_updates
                .drain(0..self.next_pending_updates.len()),
        )
    }
    pub fn run_until_stable(&mut self, max: usize) -> Result<usize, ()> {
        for i in 0..max {
            if self.pending_updates.is_empty() {
                return Ok(i);
            }
            self.tick();
        }
        Err(())
    }

    // Optimizations
    fn optimize(&mut self) {
        let old_len = self.len();
        self.const_propagation_pass();
        println!(
            "Optimized const propagation, old size:{}, new size:{}, reduction: {:.1}%",
            old_len,
            self.len(),
            (old_len - self.len()) as f64 / old_len as f64 * 100f64
        );

        let old_len = self.len();
        self.dead_code_elimination_pass();
        println!(
            "Optimized dead code elimination, old size:{}, new size:{}, reduction: {:.1}%",
            old_len,
            self.len(),
            (old_len - self.len()) as f64 / old_len as f64 * 100f64
        );

        let old_len = self.len();
        self.duplicate_dependency_pass();
        println!(
            "Optimized duplicate dependency, old size:{}, new size:{}, reduction: {:.1}%",
            old_len,
            self.len(),
            (old_len - self.len()) as f64 / old_len as f64 * 100f64
        );

        let old_len = self.len();
        self.const_propagation_pass();
        println!(
            "Optimized const propagation, old size:{}, new size:{}, reduction: {:.1}%",
            old_len,
            self.len(),
            (old_len - self.len()) as f64 / old_len as f64 * 100f64
        );
        /*
        let old_len = self.len();
        self.lut_replacement_pass();
        println!(
            "Optimized lut_replacement, old size:{}, new size:{}, reduction: {:.1}%",
            old_len,
            self.len(),
            (old_len - self.len()) as f64 / old_len as f64 * 100f64
        );

        let old_len = self.len();
        self.dead_code_elimination_pass();
        println!(
            "Optimized dead code elimination, old size:{}, new size:{}, reduction: {:.1}%",
            old_len,
            self.len(),
            (old_len - self.len()) as f64 / old_len as f64 * 100f64
        );
        */
    }
    fn find_replacement(
        &mut self,
        idx: usize,
        on: bool,
        from_const: bool,
        short_circuit: GateIndex,
        negated: bool,
    ) -> Option<GateIndex> {
        let short_circuit_output = if negated {
            short_circuit
                .opposite_if_const()
                .expect("short_circuit should be const")
        } else {
            short_circuit
        };

        if on == short_circuit.is_on() {
            return Some(short_circuit_output);
        }
        let dependencies_len = self.nodes.get(idx).unwrap().dependencies.len();
        if dependencies_len == 1 {
            if from_const {
                return Some(short_circuit_output.opposite_if_const().unwrap());
            }
            if negated {
                self.nodes.get_mut(idx).unwrap().ty = Not;
                return None;
            }
            return Some(self.nodes.get(idx).unwrap().dependencies[0]);
        }

        let mut non_const_dependency = None;
        for (i, dependency) in self
            .nodes
            .get(idx)
            .unwrap()
            .dependencies
            .iter()
            .copied()
            .enumerate()
        {
            if dependency == short_circuit_output {
                return Some(short_circuit_output);
            }
            if !dependency.is_const() {
                non_const_dependency = Some((dependency, i))
            }
        }
        if let Some((non_const_dependency, i)) = non_const_dependency {
            if dependencies_len == 2 {
                if negated {
                    let gate = self.nodes.get_mut(idx).unwrap();
                    gate.ty = Not;
                    gate.dependencies.remove(i + 1 % 2);
                    return None;
                } else {
                    return Some(non_const_dependency);
                }
            }
            return None;
        } else {
            // If there are only const dependencies and none of them are short circuits
            // the output must be the opposite of the short_circuit output.
            Some(short_circuit_output.opposite_if_const().unwrap())
        }
    }
    fn find_replacement_xor(
        &mut self,
        idx: usize,
        on: bool,
        from_const: bool,
        negated: bool,
    ) -> Option<GateIndex> {
        let dependencies_len = self.nodes.get(idx).unwrap().dependencies.len();
        if dependencies_len == 1 {
            if from_const {
                return Some(if negated ^ on { OFF } else { ON });
            }
            if negated ^ on {
                self.nodes.get_mut(idx).unwrap().ty = Not;
                return None;
            }
            return Some(self.nodes.get(idx).unwrap().dependencies[0]);
        }

        let mut non_const_dependency = None;
        let mut output = negated;
        for (i, dependency) in self
            .nodes
            .get(idx)
            .unwrap()
            .dependencies
            .iter()
            .copied()
            .enumerate()
        {
            if dependency.is_const() {
                output = output ^ dependency.is_on()
            } else {
                non_const_dependency = Some((dependency, i))
            }
        }
        if let Some((non_const_dependency, i)) = non_const_dependency {
            if dependencies_len == 2 {
                if negated ^ on {
                    let gate = self.nodes.get_mut(idx).unwrap();
                    gate.ty = Not;
                    gate.dependencies.remove(i + 1 % 2);
                    return None;
                } else {
                    return Some(non_const_dependency);
                }
            }
            return None;
        }
        Some(if output { ON } else { OFF })
    }

    // Traverses the graph forwards from constants and nodes with a single input,
    // replacing them with simpler subgraphs.
    fn const_propagation_pass(&mut self) {
        // Allocated outside main loop.
        let mut temp_dependents = Vec::new();
        let mut temp_dependencies = Vec::new();

        struct WorkItem {
            idx: usize,
            on: bool,
            from_const: bool,
        }

        // Propagate constants.
        let off = self.nodes.get_mut(OFF.idx).unwrap();

        let mut work: Vec<_> = off
            .dependents
            .drain(0..off.dependents.len())
            .map(|idx| WorkItem {
                idx,
                on: false,
                from_const: true,
            })
            .collect();

        let on = self.nodes.get_mut(ON.idx).unwrap();

        work.extend(
            on.dependents
                .drain(0..on.dependents.len())
                .map(|idx| WorkItem {
                    idx,
                    on: true,
                    from_const: true,
                }),
        );

        work.extend(self.nodes.iter().filter_map(|(idx, gate)| {
            if gate.dependencies.len() == 1 && !gate.dependencies[0].is_const() {
                return Some(WorkItem {
                    idx,
                    on: gate.ty.init(),
                    from_const: false,
                });
            }
            None
        }));

        // Seems like a reasonable heuristic.
        temp_dependents.reserve(work.len() / 2);
        temp_dependencies.reserve(work.len() / 2);

        while let Some(WorkItem {
            idx,
            on,
            from_const,
        }) = work.pop()
        {
            // Don't optimize out observable things.
            if self.is_observable(gi!(idx)) {
                continue;
            }
            if self.nodes.get(idx).is_none() {
                continue;
            }

            let gate_type = &self.nodes.get(idx).unwrap().ty;
            let replacement = match gate_type {
                Off | On | Lever => unreachable!("Off, On, and lever nodes have no dependencies"),
                Lut(..) => None,
                Not => {
                    if from_const {
                        Some(if on { OFF } else { ON })
                    } else {
                        None
                    }
                }
                And => self.find_replacement(idx, on, from_const, OFF, false),
                Nand => self.find_replacement(idx, on, from_const, OFF, true),
                Or => self.find_replacement(idx, on, from_const, ON, false),
                Nor => self.find_replacement(idx, on, from_const, ON, true),
                Xor => self.find_replacement_xor(idx, on, from_const, false),
                Xnor => self.find_replacement_xor(idx, on, from_const, true),
            };
            if let Some(replacement) = replacement {
                temp_dependents.extend(self.nodes.get(idx).unwrap().dependents.iter());
                temp_dependencies.extend(self.nodes.get(idx).unwrap().dependencies.iter().copied());

                for dependency in temp_dependencies.drain(0..temp_dependencies.len()) {
                    let dependency_dependents =
                        &mut self.nodes.get_mut(dependency.idx).unwrap().dependents;
                    dependency_dependents.remove(&idx);
                }

                if replacement.is_const() {
                    work.extend(temp_dependents.iter().copied().map(|idx| WorkItem {
                        idx,
                        on: replacement.is_on(),
                        from_const: true,
                    }))
                }

                for dependent in temp_dependents.drain(0..temp_dependents.len()) {
                    // A gate can have the same dependency many times in different dependency indexes.
                    let positions = self
                        .nodes
                        .get(dependent)
                        .unwrap()
                        .dependencies
                        .iter()
                        .enumerate()
                        .fold(
                            SmallVec::<[usize; 2]>::new(),
                            |mut acc, (position, index)| {
                                if index.idx == idx {
                                    acc.push(position)
                                }
                                acc
                            },
                        );
                    for position in positions {
                        self.nodes.get_mut(dependent).unwrap().dependencies[position] = replacement
                    }
                    self.nodes
                        .get_mut(replacement.idx)
                        .unwrap()
                        .dependents
                        .insert(dependent);
                }

                self.nodes.remove(idx);
            }
        }
    }
    // Traverses the graph backwards removing all nodes with no dependents.
    fn dead_code_elimination_pass(&mut self) {
        let mut temp_dependencies = Vec::new();

        let mut work: Vec<_> = self
            .nodes
            .iter()
            .filter_map(|(idx, gate)| {
                let idx = gi!(idx);
                if !idx.is_const() && gate.dependents.is_empty() {
                    return Some(idx);
                }
                None
            })
            .collect();
        temp_dependencies.reserve(work.len());

        while let Some(idx) = work.pop() {
            // Don't optimize out observable things.
            if self.is_observable(idx) {
                continue;
            }
            temp_dependencies.extend(
                self.nodes
                    .get(idx.idx)
                    .unwrap()
                    .dependencies
                    .iter()
                    .copied(),
            );

            for dependency in temp_dependencies.drain(0..temp_dependencies.len()) {
                let dependency_gate = self.nodes.get_mut(dependency.idx).unwrap();
                dependency_gate.dependents.remove(&idx.idx);
                if dependency_gate.dependents.is_empty() {
                    work.push(dependency)
                }
            }
            self.nodes.remove(idx.idx);
        }
    }
    // Removes duplicate dependencies from most gates
    // If the gate is an Xor or Xnor it keeps 1 if there is an odd number of copiesa.
    // or 2 if there is an even number of copies.
    fn duplicate_dependency_pass(&mut self) {
        struct WorkItem {
            idx: GateIndex,
            duplicates: SmallVec<[(GateIndex, usize); 2]>,
        }

        let mut work: Vec<WorkItem> = self
            .nodes
            .iter()
            .filter_map(|(idx, gate)| {
                let mut dependency_multi_map = HashMap::<GateIndex, usize>::new();
                // Detect duplicate dependencies and how many times they are duplicated.
                for dependency in gate.dependencies.iter().copied() {
                    let entry = dependency_multi_map.entry(dependency).or_default();
                    *entry = *entry + 1
                }

                let idx = gi!(idx);
                let duplicates: SmallVec<_> = dependency_multi_map
                    .into_iter()
                    .filter(|(_, count)| *count > 1)
                    .collect();
                if duplicates.is_empty() {
                    None
                } else {
                    Some(WorkItem { idx, duplicates })
                }
            })
            .collect();

        enum Action {
            Keep1,
            Keep2,
        }
        while let Some(WorkItem { idx, duplicates }) = work.pop() {
            // Don't optimize out observable things.
            let gate_dependencies = &mut self.nodes.get_mut(idx.idx).unwrap().dependencies;
            gate_dependencies.sort();
            gate_dependencies.dedup();
            for (duplicate, count) in duplicates {
                let gate_type = &self.nodes.get(idx.idx).unwrap().ty;
                let action = match gate_type {
                    Off | On | Lever => {
                        unreachable!("Off, On, and lever nodes have no dependencies")
                    }
                    Lut(..) => unreachable!("Rom pass should be after duplicate pass"),
                    Not => unreachable!("Not gates only have 1 dependency"),

                    And | Nand | Or | Nor => Action::Keep1,
                    Xor | Xnor => {
                        if count % 2 == 0 {
                            Action::Keep2
                        } else {
                            Action::Keep1
                        }
                    }
                };
                if let Action::Keep2 = action {
                    self.nodes
                        .get_mut(idx.idx)
                        .unwrap()
                        .dependencies
                        .push(duplicate)
                }
            }
        }
    }
    // Traverses the graph backwards from nodes with no dependants removing absorbing dependencies
    // and replacing them with a look up table.
    // It's not that easy... WIP, doing research
    fn lut_replacement_pass(&mut self) {
        let mut temp_dependencies = Vec::new();

        let mut work: Vec<_> = self
            .nodes
            .iter()
            .filter_map(|(idx, gate)| {
                let idx = gi!(idx);
                if !idx.is_const() && gate.dependents.is_empty() {
                    return Some(idx);
                }
                None
            })
            .collect();
        temp_dependencies.reserve(work.len());

        'workloop: while let Some(idx) = work.pop() {
            temp_dependencies.clear();
            temp_dependencies.extend(
                self.nodes
                    .get(idx.idx)
                    .unwrap()
                    .dependencies
                    .iter()
                    .copied(),
            );
            let ty = self.nodes.get(idx.idx).unwrap().ty.clone();
            let mut new_dependencies = SmallVec::<[GateIndex; GATE_TINYVEC_SIZE]>::new();
            let mut lut: BitVec = BitVec::new();

            let subgraph = &mut GateGraph::new();
            let subgraph_root_idx = subgraph.gate(ty, "subgraph");
            let subgraph_output = subgraph.output1(subgraph_root_idx, "subgraph-out");
            let mut subgraph_levers = Vec::<GateIndex>::new();

            'dependency_loop: for dependency in temp_dependencies.iter() {
                if *dependency == idx {
                    continue 'workloop;
                }
                let dependency_gate = self.nodes.get(dependency.idx).unwrap();
                if dependency_gate.ty.is_lever()
                    || new_dependencies.len() + dependency_gate.dependencies.len() > 4
                {
                    let lever = subgraph.lever("subgraph");
                    subgraph.dpush(subgraph_root_idx, lever);
                    subgraph_levers.push(lever);
                    if subgraph_levers.len() > std::mem::size_of::<usize>() * 8 {
                        continue 'workloop;
                    }
                    new_dependencies.push(*dependency);
                    continue 'dependency_loop;
                }
                new_dependencies.extend(dependency_gate.dependencies.iter().copied());

                let dependency_ty = dependency_gate.ty.clone();
                let dep_subgraph_idx = subgraph.gate(dependency_ty, "subgraph-deps");
                subgraph.dpush(subgraph_root_idx, dep_subgraph_idx);

                for _ in 0..dependency_gate.dependencies.len() {
                    let lever = subgraph.lever("subgraph");
                    subgraph.dpush(dep_subgraph_idx, lever);
                    subgraph_levers.push(lever);
                }
            }
            let lut_width = 1 << subgraph_levers.len();
            lut.reserve(lut_width);
            subgraph.init_unoptimized();

            println!("HI{}", lut_width);
            for i in 0..lut_width {
                subgraph.update_levers(&subgraph_levers, BitIter::new(i));
                subgraph.run_until_stable(10).unwrap();
                lut.push(subgraph_output.b0(subgraph));
            }
            println!("ho");
            for dependency in &temp_dependencies {
                self.nodes
                    .get_mut(dependency.idx)
                    .unwrap()
                    .dependents
                    .remove(&idx.idx);
            }
            for dependency in &new_dependencies {
                self.nodes
                    .get_mut(dependency.idx)
                    .unwrap()
                    .dependents
                    .insert(idx.idx);
            }
            let gate = self.nodes.get_mut(idx.idx).unwrap();
            gate.dependencies = new_dependencies;
            gate.ty = GateType::Lut(lut);
        }
    }

    // Input operations.
    fn update_lever_inner(&mut self, lever: GateIndex, value: bool) {
        assert!(
            self.nodes
                .get(lever.idx)
                .map(|l| l.ty.is_lever())
                .unwrap_or(false),
            "NodeIndex {} is not a lever",
            lever
        );
        if self.state.get_state(lever) != value {
            self.state.set(lever, value);
            self.pending_updates.push(lever);
        }
    }
    pub fn update_levers<I: Iterator<Item = bool>>(&mut self, levers: &[GateIndex], values: I) {
        for (lever, value) in levers.iter().zip(values) {
            self.update_lever_inner(*lever, value);
        }
        self.tick()
    }
    pub fn update_lever(&mut self, lever: GateIndex, value: bool) {
        self.update_lever_inner(lever, value);
        self.tick()
    }
    pub fn set_lever(&mut self, lever: GateIndex) {
        self.update_lever(lever, true)
    }
    pub fn reset_lever(&mut self, lever: GateIndex) {
        self.update_lever(lever, false)
    }
    pub fn flip_lever(&mut self, lever: GateIndex) {
        assert!(
            self.nodes
                .get(lever.idx)
                .map(|l| l.ty.is_lever())
                .unwrap_or(false),
            "NodeIndex {} is not a lever",
            lever
        );

        self.state.set(lever, !self.state.get_state(lever));
        self.pending_updates.push(lever);
        self.tick();
    }
    pub fn pulse_lever(&mut self, lever: GateIndex) {
        self.set_lever(lever);
        self.reset_lever(lever);
    }

    pub fn set_lever_stable(&mut self, lever: GateIndex) {
        self.set_lever(lever);
        self.run_until_stable(10).unwrap();
    }
    pub fn reset_lever_stable(&mut self, lever: GateIndex) {
        self.reset_lever(lever);
        self.run_until_stable(10).unwrap();
    }
    pub fn flip_lever_stable(&mut self, lever: GateIndex) {
        self.flip_lever(lever);
        self.run_until_stable(10).unwrap();
    }
    pub fn pulse_lever_stable(&mut self, lever: GateIndex) {
        self.set_lever(lever);
        self.run_until_stable(10).unwrap();
        self.reset_lever(lever);
        self.run_until_stable(10).unwrap();
    }

    // Output operations.
    fn is_observable(&self, gate: GateIndex) -> bool {
        if self.outputs.contains(&gate) {
            return true;
        }
        #[cfg(feature = "debug_gate_names")]
        if self.probes.contains_key(&gate) {
            return true;
        }
        false
    }
    pub fn output<S: Into<String>>(&mut self, bits: &[GateIndex], name: S) -> CircuitOutput {
        for bit in bits {
            self.outputs.insert(*bit);
        }
        CircuitOutput {
            bits: bits.into(),
            name: name.into(),
        }
    }
    pub fn output1<S: Into<String>>(&mut self, bit: GateIndex, name: S) -> CircuitOutput {
        self.outputs.insert(bit);
        CircuitOutput {
            bits: smallvec![bit],
            name: name.into(),
        }
    }
    fn value(&self, idx: GateIndex) -> bool {
        self.state.get_state(idx)
    }
    // Collect only first 8 bits from a larger bus.
    // Or only some bits from a smaller bus.
    fn collect_u8_lossy(&self, outputs: &[GateIndex]) -> u8 {
        let mut output = 0;
        let mut mask = 1u8;

        for bit in outputs.iter().take(8) {
            if self.value(*bit) {
                output = output | mask
            }

            mask = mask << 1;
        }

        output
    }
    fn collect_usize_lossy(&self, outputs: &[GateIndex]) -> usize {
        let mut output = 0;
        let mut mask = 1usize;
        let usize_width = std::mem::size_of::<usize>() * 8;

        for bit in outputs.iter().take(usize_width) {
            if self.value(*bit) {
                output = output | mask
            }

            mask = mask << 1;
        }

        output
    }
    // Collect only first 128 bits from a larger bus.
    // Or only some bits from a smaller bus.
    fn collect_u128_lossy(&self, outputs: &[GateIndex]) -> u128 {
        let mut output = 0;
        let mut mask = 1u128;

        for bit in outputs.iter().take(128) {
            if self.value(*bit) {
                output = output | mask
            }

            mask = mask << 1;
        }

        output
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn dump_dot(&self, filename: &Path) {
        use petgraph::dot::{Config, Dot};
        use std::io::Write;
        let mut f = std::fs::File::create(filename).unwrap();
        let mut graph = petgraph::Graph::<_, ()>::new();
        let mut index = HashMap::new();
        for (i, node) in self.nodes.iter() {
            let is_out = self.outputs.contains(&gi!(i));
            #[cfg(feature = "debug_gate_names")]
            let name = self
                .names
                .get(&gi!(i))
                .map(|name| format!(":{}", name))
                .unwrap_or("".to_string());

            #[cfg(not(feature = "debug_gate_names"))]
            let label = if is_out {
                format!("output:{}", node.ty)
            } else {
                node.ty.to_string()
            };
            #[cfg(feature = "debug_gate_names")]
            let label = if is_out {
                format!("O:{}{}", node.ty, name)
            } else {
                format!("{}{}", node.ty, name)
            };
            index.insert(i, graph.add_node(label));
        }
        for (i, node) in self.nodes.iter() {
            graph.extend_with_edges(
                node.dependencies
                    .iter()
                    .map(|dependency| (index[&dependency.idx], index[&i])),
            );
        }
        write!(f, "{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel])).unwrap();
    }

    // Debug operations.
    #[cfg(feature = "debug_gate_names")]
    pub fn probe<S: Into<String>>(&mut self, bits: &[GateIndex], name: S) {
        let name = name.into();
        for bit in bits {
            self.probes.insert(
                *bit,
                Probe {
                    name: name.clone(),
                    bits: SmallVec::from_slice(bits),
                },
            );
        }
    }
    #[cfg(feature = "debug_gate_names")]
    pub fn probe1<S: Into<String>>(&mut self, bit: GateIndex, name: S) {
        self.probe(&[bit], name)
    }

    // Test operations.
    #[cfg(test)]
    pub fn assert_propagation(&mut self, expected: usize) {
        let actual = self
            .run_until_stable(1000)
            .expect("Circuit didn't stabilize after 1000 ticks");

        assert!(
            actual == expected,
            "Circuit stabilized after {} ticks, expected: {}",
            actual,
            expected
        );
    }
    #[cfg(test)]
    pub fn assert_propagation_range(&mut self, expected: std::ops::Range<usize>) {
        let actual = self
            .run_until_stable(1000)
            .expect("Circuit didn't stabilize after 1000 ticks");

        assert!(
            expected.contains(&actual),
            "Circuit stabilized after {} ticks, which is outside the range: {}..{}",
            actual,
            expected.start,
            expected.end
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flip_flop() {
        let g = &mut GateGraph::new();

        let set = g.lever("");
        let reset = g.lever("");

        let flip = g.or2(reset, OFF, "");
        let q = g.not1(flip, "");

        let flop = g.or2(set, q, "");
        let nq = g.not1(flop, "");
        g.d1(flip, nq);

        let output = g.output1(nq, "nq");
        g.init();

        g.run_until_stable(10).unwrap();
        for _ in 0..10 {
            assert_eq!(output.b0(g), true);
        }
        println!("b4lever");
        g.update_lever(set, true);
        println!("aftlever");

        g.run_until_stable(10).unwrap();
        assert_eq!(output.b0(g), false);

        g.update_lever(set, false);

        g.run_until_stable(10).unwrap();
        assert_eq!(output.b0(g), false);
    }
    #[test]
    fn test_not_loop() {
        let g = &mut GateGraph::new();
        let n1 = g.not("n1");
        let n2 = g.not1(n1, "n2");
        let n3 = g.not1(n2, "n3");
        g.d0(n1, n3);

        let output = g.output1(n1, "n1");
        g.init();

        let mut a = true;
        for _ in 0..10 {
            assert_eq!(output.b0(g), a);
            g.tick();
            a = !a;
        }

        // There is no stable state
        assert!(g.run_until_stable(100).is_err())
    }
    #[test]
    fn test_big_and() {
        let g = &mut GateGraph::new();
        let and = g.and2(ON, ON, "and");
        let output = g.output(&[and], "big_and");
        g.dpush(and, ON);
        g.dpush(and, ON);
        g.init();

        assert_eq!(output.b0(g), true)
    }
}
