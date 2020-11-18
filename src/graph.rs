use crate::slab::Slab;
use crate::state::State;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
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
}
impl BaseNodeGraph {
    pub fn new() -> BaseNodeGraph {
        BaseNodeGraph { nodes: Slab::new() }
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
    pub fn value(&self, idx: usize, state: &mut State) -> Option<bool> {
        let node = self.nodes.get(idx)?;
        Some(if state.get_updated(idx) {
            state.get_state(idx)
        } else {
            match node.deref() {
                Lever => state.get_state(idx),
                Not { dep } => {
                    let new_state = !state.get_state(*dep);
                    state.set(idx, new_state);
                    // update state
                    self.value(*dep, state)?;
                    new_state
                }
                Xor { deps } => {
                    let a = state.get_state(deps[0]);
                    let b = state.get_state(deps[1]);

                    let new_state = a ^ b;

                    state.set(idx, new_state);

                    // update states
                    self.value(deps[0], state);
                    self.value(deps[1], state);
                    new_state
                }
                Or { deps } => {
                    let res = deps.iter().try_fold(false, |acc, dep| {
                        let old_state = state.get_state(*dep);
                        Some(acc || old_state)
                    })?;

                    state.set(idx, res);

                    // update states
                    for dep in deps.iter() {
                        self.value(*dep, state)?;
                    }
                    res
                }
                And { deps } => {
                    let res = deps.iter().clone().try_fold(true, |acc, dep| {
                        let old_state = state.get_state(*dep);
                        Some(acc && old_state)
                    })?;

                    state.set(idx, res);

                    // update states
                    for dep in deps.iter() {
                        self.value(*dep, state)?;
                    }

                    res
                }
            }
        })
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}
