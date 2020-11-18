use crate::slab::Slab;
use crate::state::State;
use std::ops::Deref;
pub enum BaseNode {
    Lever,
    Or { deps: [usize; 2] },
    And { deps: [usize; 2] },
    Xor { deps: [usize; 2] },
    Not { dep: usize },
}
pub const OFF: usize = std::usize::MAX;
pub const ON: usize = std::usize::MAX - 1;
use BaseNode::*;

pub struct BaseNodeGraph {
    nodes: Slab<BaseNode>,
    scrap_updates_stack: Vec<usize>, // allocated outside to prevent allocations in the hot loop.
}
impl BaseNodeGraph {
    pub fn new() -> BaseNodeGraph {
        BaseNodeGraph {
            nodes: Slab::new(),
            scrap_updates_stack: Vec::new(),
        }
    }
    pub fn not(&mut self) -> Option<usize> {
        self.nodes.insert(Not { dep: OFF })
    }
    pub fn not1(&mut self, dep: usize) -> Option<usize> {
        self.nodes.insert(Not { dep })
    }
    pub fn or(&mut self) -> Option<usize> {
        self.nodes.insert(Or { deps: [OFF, OFF] })
    }
    pub fn lever(&mut self) -> Option<usize> {
        self.nodes.insert(Lever)
    }
    pub fn or2(&mut self, d1: usize, d2: usize) -> Option<usize> {
        self.nodes.insert(Or { deps: [d1, d2] })
    }
    pub fn xor2(&mut self, d1: usize, d2: usize) -> Option<usize> {
        self.nodes.insert(Xor { deps: [d1, d2] })
    }
    pub fn and2(&mut self, d1: usize, d2: usize) -> Option<usize> {
        self.nodes.insert(And { deps: [d1, d2] })
    }
    fn update(&mut self, idx: usize, state: &mut State) {
        self.scrap_updates_stack.push(idx);
        while let Some(idx) = self.scrap_updates_stack.pop() {
            let node = self.nodes.get(idx).unwrap();
            match node.deref() {
                Lever => {}
                Not { dep } => {
                    let new_state = !state.get_state(*dep);
                    state.set(idx, new_state);
                    // update state
                    if !state.get_updated(*dep) {
                        self.scrap_updates_stack.push(*dep)
                    }
                }
                v @ Xor { deps } | v @ Or { deps } | v @ And { deps } => {
                    let a = state.get_state(deps[0]);
                    let b = state.get_state(deps[1]);

                    let new_state = match v {
                        Xor { .. } => a ^ b,
                        Or { .. } => a || b,
                        And { .. } => a && b,
                        _ => unreachable!(),
                    };

                    state.set(idx, new_state);

                    // update states
                    if !state.get_updated(deps[0]) {
                        self.scrap_updates_stack.push(deps[0])
                    }
                    if !state.get_updated(deps[1]) {
                        self.scrap_updates_stack.push(deps[1])
                    }
                }
            }
        }
    }
    pub fn value(&mut self, idx: usize, state: &mut State) -> bool {
        if !state.get_updated(idx) {
            self.update(idx, state);
        }
        state.get_state(idx)
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}
