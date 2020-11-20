use crate::slab::Slab;
use crate::state::State;
use smallvec::{smallvec, SmallVec};
use std::collections::HashSet;
use std::collections::{HashMap, VecDeque};
use std::convert::TryInto;
use std::fmt::{self, Display, Formatter};
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct NodeIndex {
    pub idx: usize,
}
impl Display for NodeIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.idx)
    }
}
macro_rules! ni {
    ( $x:expr ) => {{
        NodeIndex::new($x)
    }};
}
pub const OFF: NodeIndex = ni!(0);
pub const ON: NodeIndex = ni!(1);

impl NodeIndex {
    pub const fn new(idx: usize) -> NodeIndex {
        NodeIndex { idx }
    }
    pub fn is_off(&self) -> bool {
        *self == OFF
    }
    pub fn is_on(&self) -> bool {
        *self == ON
    }
}
#[derive(Copy, Clone, Debug)]
pub enum BaseNode {
    Off,
    On,
    Lever,
    Xor,
    Not,
    Or,
    And,
    Nand,
    Nor,
}
impl BaseNode {
    fn accumulate(&self, acc: bool, b: bool) -> bool {
        match self {
            Or | Nor => acc || b,
            And | Nand => acc && b,
            On | Off | Lever | Not | Xor => unreachable!(),
        }
    }
    fn init(&self) -> bool {
        match self {
            Or | Nor => false,
            And | Nand => true,
            On | Off | Lever | Not | Xor => unreachable!(),
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
        if let Nor | Nand | Not = self {
            true
        } else {
            false
        }
    }
}
use BaseNode::*;
#[derive(Clone)]
struct Gate {
    ty: BaseNode,
    dependencies: SmallVec<[NodeIndex; 4]>,
    dependents: HashSet<NodeIndex>,
}
impl Gate {
    fn new(ty: BaseNode, dependencies: SmallVec<[NodeIndex; 4]>) -> Self {
        Gate {
            ty,
            dependencies,
            dependents: HashSet::new(),
        }
    }
}

pub struct BaseNodeGraph {
    nodes: Slab<Gate>,
    update_sources: Vec<NodeIndex>,
    next_update_sources: Vec<NodeIndex>,
    scrap_updates_queue: VecDeque<NodeIndex>, // allocated outside to prevent allocations in the hot loop.
    state: State,
    #[cfg(feature = "debug_gate_names")]
    names: HashMap<NodeIndex, String>,
}
impl BaseNodeGraph {
    pub fn new() -> BaseNodeGraph {
        let mut nodes = Slab::new();
        nodes.insert(Gate {
            ty: Off,
            dependencies: smallvec![],
            dependents: HashSet::new(),
        });
        nodes.insert(Gate {
            ty: On,
            dependencies: smallvec![],
            dependents: HashSet::new(),
        });
        BaseNodeGraph {
            nodes,
            update_sources: vec![],
            next_update_sources: vec![],
            state: State::new(),
            scrap_updates_queue: VecDeque::new(),
            #[cfg(feature = "debug_gate_names")]
            names: HashMap::new(),
        }
    }

    // Dependency operations.
    pub fn dpush(&mut self, idx: NodeIndex, new_dep: NodeIndex) {
        let gate = self.nodes.get_mut(idx.idx).unwrap();
        match gate.ty {
            Off => assert!(false, "OFF has no dependencies"),
            On => assert!(false, "ON has no dependencies"),
            Lever => assert!(false, "Lever has no dependencies"),
            Not => assert!(false, "Not has fixed dependencies"),
            Xor => assert!(false, "Xor has fixed dependencies"),
            Or | And | Nand | Nor => {
                gate.dependencies.push(new_dep);
                self.nodes
                    .get_mut(new_dep.idx)
                    .unwrap()
                    .dependents
                    .insert(idx);
            }
        }
    }
    pub fn dx(&mut self, idx: NodeIndex, new_dep: NodeIndex, x: usize) {
        let gate = self.nodes.get_mut(idx.idx).unwrap();
        match gate.ty {
            Off => assert!(false, "OFF has no dependencies"),
            On => assert!(false, "ON has no dependencies"),
            Lever => assert!(false, "Lever has no dependencies"),
            Not => {
                assert!(x == 0, "Not only has one dependency");
            }
            Xor => {
                assert!(x < 2, "Xor has only 2 dependencies");
            }
            Or | And | Nand | Nor => {}
        }

        let old_dep = std::mem::replace(&mut gate.dependencies[x], new_dep);

        self.nodes
            .get_mut(old_dep.idx)
            .unwrap()
            .dependents
            .remove(&idx);
        self.nodes
            .get_mut(new_dep.idx)
            .unwrap()
            .dependents
            .insert(idx);
    }
    pub fn d0(&mut self, gate: NodeIndex, dep: NodeIndex) {
        self.dx(gate, dep, 0)
    }
    pub fn d1(&mut self, gate: NodeIndex, dep: NodeIndex) {
        self.dx(gate, dep, 1)
    }

    // Gate operations.
    #[inline(always)]
    fn create_gate<S: Into<String>>(&mut self, idx: NodeIndex, deps: &[NodeIndex], name: S) {
        for dep in deps {
            self.nodes.get_mut(dep.idx).unwrap().dependents.insert(idx);
        }
        if cfg!(feature = "debug_gate_names") {
            self.names.insert(idx, name.into());
        }
    }
    pub fn lever<S: Into<String>>(&mut self, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Gate::new(Lever, smallvec![])));
        self.create_gate(idx, &[], name);
        idx
    }
    pub fn not<S: Into<String>>(&mut self, name: S) -> NodeIndex {
        self.not1(OFF, name)
    }
    pub fn not1<S: Into<String>>(&mut self, dep: NodeIndex, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Gate::new(Not, smallvec![dep])));
        self.create_gate(idx, &[dep], name);
        idx
    }
    pub fn or<S: Into<String>>(&mut self, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Gate::new(Or, smallvec![])));
        self.create_gate(idx, &[], name);
        idx
    }
    pub fn or2<S: Into<String>>(&mut self, d0: NodeIndex, d1: NodeIndex, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Gate::new(Or, smallvec![d0, d1])));
        self.create_gate(idx, &[d0, d1], name);
        idx
    }
    pub fn nor2<S: Into<String>>(&mut self, d0: NodeIndex, d1: NodeIndex, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Gate::new(Nor, smallvec![d0, d1])));
        self.create_gate(idx, &[d0, d1], name);
        idx
    }
    pub fn xor2<S: Into<String>>(&mut self, d0: NodeIndex, d1: NodeIndex, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Gate::new(Xor, smallvec![d0, d1])));
        self.create_gate(idx, &[d0, d1], name);
        idx
    }
    pub fn and<S: Into<String>>(&mut self, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Gate::new(And, smallvec![])));
        self.create_gate(idx, &[], name);
        idx
    }
    pub fn and2<S: Into<String>>(&mut self, d0: NodeIndex, d1: NodeIndex, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Gate::new(And, smallvec![d0, d1])));
        self.create_gate(idx, &[d0, d1], name);
        idx
    }
    pub fn nand2<S: Into<String>>(&mut self, d0: NodeIndex, d1: NodeIndex, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Gate::new(Nand, smallvec![d0, d1])));
        self.create_gate(idx, &[d0, d1], name);
        idx
    }

    // Main logic.
    fn update(&mut self) {
        while let Some(pending) = &self.update_sources.pop() {
            self.state.tick();
            self.scrap_updates_queue.push_back(*pending);
            while let Some(idx) = self.scrap_updates_queue.pop_front() {
                let node = self.nodes.get(idx.idx).unwrap();
                let new_state = match node.ty {
                    On => true,
                    Off => false,
                    Lever => self.state.get_state(idx),
                    Not => !self.state.get_state(node.dependencies[0]),
                    Xor => {
                        self.state.get_state(node.dependencies[0])
                            ^ self.state.get_state(node.dependencies[1])
                    }
                    Or | Nand | And | Nor => {
                        if node.dependencies.is_empty() {
                            false
                        } else {
                            let init = node.ty.init();
                            let mut new_state = node
                                .dependencies
                                .iter()
                                .map(|dep| self.state.get_state(*dep))
                                .fold(init, |acc, s| node.ty.accumulate(acc, s));
                            if node.ty.is_negated() {
                                new_state = !new_state;
                            }
                            new_state
                        }
                    }
                };
                if let Some(old_state) = self.state.get_if_updated(idx) {
                    if old_state != new_state {
                        self.next_update_sources.push(idx);
                    }
                    continue;
                }
                self.state.set(idx, new_state);
                self.scrap_updates_queue.extend(node.dependents.iter())
            }
        }
        self.update_sources.extend(
            self.next_update_sources
                .drain(0..self.next_update_sources.len()),
        )
    }
    pub fn value(&mut self, idx: NodeIndex) -> bool {
        self.state.get_state(idx)
    }
    pub fn init(&mut self) {
        self.state.reserve(self.len());

        for (idx, _) in self.nodes.iter() {
            if ni!(idx) != OFF && ni!(idx) != ON && self.state.get_updated(ni!(idx)) {
                continue;
            }
            self.scrap_updates_queue.push_back(ni!(idx));
            while let Some(idx) = self.scrap_updates_queue.pop_front() {
                let node = self.nodes.get(idx.idx).unwrap();
                let new_state = match node.ty {
                    On => true,
                    Off => false,
                    Lever => self.state.get_state(idx),
                    Not => !self.state.get_state(node.dependencies[0]),
                    Xor => {
                        self.state.get_state(node.dependencies[0])
                            ^ self.state.get_state(node.dependencies[1])
                    }
                    Or | Nand | And | Nor => {
                        if node.dependencies.is_empty() {
                            false
                        } else {
                            let init = node.ty.init();
                            let mut new_state = node
                                .dependencies
                                .iter()
                                .map(|dep| self.state.get_state(*dep))
                                .fold(init, |acc, s| node.ty.accumulate(acc, s));
                            if node.ty.is_negated() {
                                new_state = !new_state;
                            }
                            new_state
                        }
                    }
                };
                if let Some(old_state) = self.state.get_if_updated(idx) {
                    if old_state != new_state {
                        self.update_sources.push(idx);
                    }
                    continue;
                }
                self.state.set(idx, new_state);
                self.scrap_updates_queue.extend(node.dependents.iter())
            }
        }
    }
    pub fn run_until_stable(&mut self, max: usize) -> Result<usize, ()> {
        for i in 0..max {
            if self.update_sources.is_empty() {
                return Ok(i);
            }
            self.update();
        }
        Err(())
    }

    // Input operations.
    fn update_lever_inner(&mut self, lever: NodeIndex, value: bool) {
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
            self.update_sources.push(lever);
        }
    }
    pub fn update_levers<I: Iterator<Item = bool>>(&mut self, levers: &[NodeIndex], values: I) {
        for (lever, value) in levers.iter().zip(values) {
            self.update_lever_inner(*lever, value);
        }
        self.update()
    }
    pub fn update_lever(&mut self, lever: NodeIndex, value: bool) {
        self.update_lever_inner(lever, value);
        self.update()
    }
    pub fn set_lever(&mut self, lever: NodeIndex) {
        self.update_lever(lever, true)
    }
    pub fn reset_lever(&mut self, lever: NodeIndex) {
        self.update_lever(lever, false)
    }
    pub fn flip_lever(&mut self, lever: NodeIndex) {
        assert!(
            self.nodes
                .get(lever.idx)
                .map(|l| l.ty.is_lever())
                .unwrap_or(false),
            "NodeIndex {} is not a lever",
            lever
        );

        self.state.set(lever, !self.state.get_state(lever));
        self.update_sources.push(lever);
        self.update();
    }
    // Output operations.
    pub fn collect_u8(&mut self, outputs: &[NodeIndex; 8]) -> u8 {
        let mut output = 0;
        let mut mask = 1u8;

        for bit in outputs {
            if self.value(*bit) {
                output = output | mask
            }

            mask = mask << 1;
        }

        output
    }
    pub fn collect_u128(&mut self, outputs: &[NodeIndex; 128]) -> u128 {
        let mut output = 0;
        let mut mask = 1u128;

        for bit in outputs {
            if self.value(*bit) {
                output = output | mask
            }

            mask = mask << 1;
        }

        output
    }
    pub fn collect_u8_lossy(&mut self, outputs: &[NodeIndex]) -> u8 {
        self.collect_u8(outputs[0..8].try_into().unwrap())
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flip_flop() {
        let mut g = BaseNodeGraph::new();

        let set = g.lever("");
        let reset = g.lever("");

        let flip = g.or2(reset, OFF, "");
        let q = g.not1(flip, "");

        let flop = g.or2(set, q, "");
        let nq = g.not1(flop, "");
        g.d1(flip, nq);
        g.init();

        g.run_until_stable(10).unwrap();
        for _ in 0..10 {
            assert_eq!(g.value(nq), true);
        }
        g.update_lever(set, true);

        g.run_until_stable(10).unwrap();
        assert_eq!(g.value(nq), false);

        g.update_lever(set, false);

        g.run_until_stable(10).unwrap();
        assert_eq!(g.value(nq), false);
    }
    #[test]
    fn test_not_loop() {
        let mut g = BaseNodeGraph::new();
        let n1 = g.not("n1");
        let n2 = g.not1(n1, "name");
        let n3 = g.not1(n2, "name");
        g.d0(n1, n3);
        g.init();

        let mut a = true;
        for _ in 0..10 {
            assert_eq!(g.value(n1), a);
            g.update();
            a = !a;
        }

        // There is no stable state
        assert!(g.run_until_stable(100).is_err())
    }
    #[test]
    fn test_big_and() {
        let mut g = BaseNodeGraph::new();
        let and = g.and2(ON, ON, "and");
        g.dpush(and, ON);
        g.dpush(and, ON);
        g.init();

        assert_eq!(g.value(and), true)
    }
}
