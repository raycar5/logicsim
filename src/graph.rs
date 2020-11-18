use crate::slab::Slab;
use crate::state::State;
use std::collections::{HashMap, HashSet, VecDeque};
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct NodeIndex {
    pub idx: usize,
}
macro_rules! ni {
    ( $x:expr ) => {{
        NodeIndex::new($x)
    }};
}
pub const OFF: NodeIndex = ni!(std::usize::MAX);
pub const ON: NodeIndex = ni!(std::usize::MAX - 1);
pub const LATER: NodeIndex = ni!(std::usize::MAX - 2);
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
    pub fn is_later(&self) -> bool {
        *self == LATER
    }
}
#[derive(Copy, Clone)]
pub enum BaseNode {
    Lever,
    Or { deps: [NodeIndex; 2] },
    And { deps: [NodeIndex; 2] },
    Xor { deps: [NodeIndex; 2] },
    Nand { deps: [NodeIndex; 2] },
    Not { dep: NodeIndex },
}
use BaseNode::*;

pub struct BaseNodeGraph {
    nodes: Slab<BaseNode>,
    scrap_updates_stack: Vec<NodeIndex>, // allocated outside to prevent allocations in the hot loop.
    #[cfg(feature = "debug_gate_names")]
    names: HashMap<NodeIndex, String>,
}
impl BaseNodeGraph {
    pub fn new() -> BaseNodeGraph {
        BaseNodeGraph {
            nodes: Slab::new(),
            scrap_updates_stack: Vec::new(),
            #[cfg(feature = "debug_gate_names")]
            names: HashMap::new(),
        }
    }
    pub fn dx(&mut self, gate: NodeIndex, new_dep: NodeIndex, x: usize) {
        assert!(x < 2, "Gates have a maximum of 2 dependencies");
        match self.nodes.get_mut(gate.idx).unwrap() {
            Lever => {
                assert!(false, "Lever has no dependencies")
            }
            Not { dep } => {
                assert!(x == 0, "Not only has one dependency");
                *dep = new_dep;
            }
            Xor { deps } | Or { deps } | And { deps } | Nand { deps } => {
                deps[x] = new_dep;
            }
        }
    }
    pub fn d0(&mut self, gate: NodeIndex, dep: NodeIndex) {
        self.dx(gate, dep, 0)
    }
    pub fn d1(&mut self, gate: NodeIndex, dep: NodeIndex) {
        self.dx(gate, dep, 1)
    }
    pub fn not<S: Into<String>>(&mut self, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Not { dep: OFF }));
        if cfg!(feature = "debug_gate_names") {
            self.names.insert(idx, name.into());
        }
        idx
    }
    pub fn not1<S: Into<String>>(&mut self, dep: NodeIndex, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Not { dep }));
        if cfg!(feature = "debug_gate_names") {
            self.names.insert(idx, name.into());
        }
        idx
    }
    pub fn or<S: Into<String>>(&mut self, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Or { deps: [OFF, OFF] }));
        if cfg!(feature = "debug_gate_names") {
            self.names.insert(idx, name.into());
        }
        idx
    }
    pub fn lever<S: Into<String>>(&mut self, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Lever));
        if cfg!(feature = "debug_gate_names") {
            self.names.insert(idx, name.into());
        }
        idx
    }
    pub fn or2<S: Into<String>>(&mut self, d0: NodeIndex, d1: NodeIndex, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Or { deps: [d0, d1] }));
        if cfg!(feature = "debug_gate_names") {
            self.names.insert(idx, name.into());
        }
        idx
    }
    pub fn xor2<S: Into<String>>(&mut self, d0: NodeIndex, d1: NodeIndex, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Xor { deps: [d0, d1] }));
        if cfg!(feature = "debug_gate_names") {
            self.names.insert(idx, name.into());
        }
        idx
    }
    pub fn and2<S: Into<String>>(&mut self, d0: NodeIndex, d1: NodeIndex, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(And { deps: [d0, d1] }));
        if cfg!(feature = "debug_gate_names") {
            self.names.insert(idx, name.into());
        }
        idx
    }
    pub fn nand2<S: Into<String>>(&mut self, d0: NodeIndex, d1: NodeIndex, name: S) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.insert(Nand { deps: [d0, d1] }));
        if cfg!(feature = "debug_gate_names") {
            self.names.insert(idx, name.into());
        }
        idx
    }
    fn update(&mut self, idx: NodeIndex, state: &mut State) {
        self.scrap_updates_stack.push(idx);
        while let Some(idx) = self.scrap_updates_stack.pop() {
            if state.get_updated(idx) {
                continue;
            }
            let node = self.nodes.get(idx.idx).unwrap();
            match node {
                Lever => state.set_updated(idx),
                Not { dep } => {
                    let val = state.get_state(*dep);
                    let updated = state.get_updated(*dep);
                    if updated {
                        state.set(idx, !val)
                    } else if idx.idx < dep.idx {
                        state.set(idx, !val);
                        self.scrap_updates_stack.push(*dep);
                    } else {
                        self.scrap_updates_stack.push(idx);
                        self.scrap_updates_stack.push(*dep);
                    }
                }
                Xor { deps } | Or { deps } | Nand { deps } | And { deps } => {
                    let updated0 = state.get_updated(deps[0]);
                    let updated1 = state.get_updated(deps[1]);
                    if !updated0 && !idx.idx < deps[0].idx {
                        self.scrap_updates_stack.push(idx);
                        self.scrap_updates_stack.push(deps[0]);
                        if !updated1 && !idx.idx < deps[1].idx {
                            self.scrap_updates_stack.push(deps[1]);
                            continue;
                        }
                        continue;
                    }

                    let a = state.get_state(deps[0]);
                    let b = state.get_state(deps[1]);

                    let new_state = match node {
                        Xor { .. } => a ^ b,
                        Or { .. } => a || b,
                        And { .. } => a && b,
                        Nand { .. } => !(a && b),
                        Lever | Not { .. } => unreachable!(),
                    };

                    state.set(idx, new_state);

                    // update states
                    if !updated0 {
                        self.scrap_updates_stack.push(deps[0])
                    }
                    if !updated1 {
                        self.scrap_updates_stack.push(deps[1])
                    }
                }
            }
        }
    }
    pub fn value(&mut self, idx: NodeIndex, state: &mut State) -> bool {
        if !state.get_updated(idx) {
            self.update(idx, state);
        }
        state.get_state(idx)
    }
    pub fn init(&mut self, state: &mut State) {
        let mut work: VecDeque<_> = self.nodes.iter().map(|(i, n)| (i, *n)).collect();
        while let Some((idx, node)) = work.pop_front() {
            if state.get_updated(ni!(idx)) {
                continue;
            }
            match node {
                Lever => state.set_updated(ni!(idx)),
                Not { dep } => {
                    if let Some(val) = state.get_if_updated(dep) {
                        state.set(ni!(idx), !val)
                    } else if idx < dep.idx {
                        state.set(ni!(idx), true)
                    } else {
                        work.push_back((idx, node))
                    }
                }
                Xor { deps } | Or { deps } | Nand { deps } | And { deps } => {
                    let d0 = state.get_if_updated(deps[0]).or_else(|| {
                        if idx < deps[0].idx {
                            Some(false)
                        } else {
                            None
                        }
                    });
                    let d1 = state.get_if_updated(deps[1]).or_else(|| {
                        if idx < deps[1].idx {
                            Some(false)
                        } else {
                            None
                        }
                    });
                    if let (Some(d0), Some(d1)) = (d0, d1) {
                        let val = match node {
                            Xor { .. } => d0 ^ d1,
                            Or { .. } => d0 || d1,
                            Nand { .. } => !(d0 && d1),
                            And { .. } => d0 && d1,
                            Lever | Not { .. } => unreachable!(),
                        };
                        state.set(ni!(idx), val)
                    } else {
                        work.push_back((idx, node))
                    }
                }
            }
        }
        state.tick()
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
        let mut states = State::new(10);

        let set = g.lever("");
        let reset = g.lever("");

        let flip = g.or2(reset, LATER, "");
        let q = g.not1(flip, "");

        let flop = g.or2(set, q, "");
        let nq = g.not1(flop, "");
        g.d1(flip, nq);
        g.init(&mut states);

        for _ in 0..10 {
            assert_eq!(g.value(q, &mut states), true);
            states.tick();
        }
        states.set(reset, true);

        assert_eq!(g.value(q, &mut states), false);
        states.tick();

        assert_eq!(g.value(q, &mut states), false);
        states.tick();

        states.set(reset, false);

        for _ in 0..10 {
            assert_eq!(g.value(q, &mut states), false);
            states.tick();
            println!(
                "flip:{}, q:{}, flop:{}, nq:{}",
                states.get_state(flip),
                states.get_state(q),
                states.get_state(flop),
                states.get_state(nq)
            );
        }
    }
}
