use super::types::*;
use crate::data_structures::State;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

pub struct InitializedGateGraph {
    pub(super) nodes: Vec<Gate>,
    pub(super) pending_updates: Vec<GateIndex>,
    pub(super) next_pending_updates: Vec<GateIndex>, // Allocated outside to prevent allocations in the hot loop.
    pub(super) propagation_queue: VecDeque<GateIndex>, // Allocated outside to prevent allocations in the hot loop.
    pub(super) output_handles: Vec<CircuitOutput>,
    pub(super) lever_handles: Vec<GateIndex>,
    pub(super) outputs: HashSet<GateIndex>,
    pub(super) state: State,
    #[cfg(feature = "debug_gates")]
    pub(super) names: HashMap<GateIndex, String>,
    #[cfg(feature = "debug_gates")]
    pub(super) probes: HashMap<GateIndex, Probe>,
}
use GateType::*;
impl InitializedGateGraph {
    #[inline(always)]
    unsafe fn fold_short(&self, ty: &GateType, gates: &[GateIndex]) -> bool {
        let init = ty.init();
        let short = !init;
        // Using a manual loop results in 2% less instructions.
        #[allow(clippy::needless_range_loop)]
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
    pub(super) fn tick_inner(&mut self) {
        while let Some(idx) = self.propagation_queue.pop_front() {
            // This is safe because the propagation queue gets filled by items coming from
            // nodes.iter() or levers, both of which are always in bounds.
            debug_assert!(idx.idx < self.nodes.len());
            let node = unsafe { self.nodes.get_unchecked(idx.idx) };

            let new_state = match &node.ty {
                On => true,
                Off => false,
                // This is safe because I fill the state on init.
                Lever => unsafe { self.state.get_state_very_unsafely(idx) },
                Not => unsafe { !self.state.get_state_very_unsafely(node.dependencies[0]) },
                Or | Nor | And | Nand | Xor | Xnor => {
                    let mut new_state = if node.dependencies.is_empty() {
                        false
                    } else if node.ty.short_circuits() {
                        // This is safe because I fill the state on init.
                        unsafe { self.fold_short(&node.ty, &node.dependencies) }
                    } else {
                        let mut result = node.ty.init();

                        // Using a manual loop results in 2% less instructions.
                        #[allow(clippy::needless_range_loop)]
                        for i in 0..node.dependencies.len() {
                            // This is safe because I fill the state on init.
                            let state =
                                unsafe { self.state.get_state_very_unsafely(node.dependencies[i]) };
                            result = node.ty.accumulate(result, state);
                        }
                        result
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

            #[cfg(feature = "debug_gates")]
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
                    .extend(node.dependents.iter().copied())
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
    pub fn run_until_stable(&mut self, max: usize) -> Result<usize, &'static str> {
        for i in 0..max {
            if self.pending_updates.is_empty() {
                return Ok(i);
            }
            self.tick();
        }
        Err("Your graph didn't stabilize")
    }
    // Input operations.
    fn update_lever_inner(&mut self, lever: LeverHandle, value: bool) {
        let idx = self.lever_handles[lever.handle];
        if self.state.get_state(idx) != value {
            self.state.set(idx, value);
            self.pending_updates.push(idx);
        }
    }
    pub fn update_levers<I: Iterator<Item = bool>>(&mut self, levers: &[LeverHandle], values: I) {
        for (lever, value) in levers.iter().zip(values) {
            self.update_lever_inner(*lever, value);
        }
        self.tick()
    }
    pub fn update_lever(&mut self, lever: LeverHandle, value: bool) {
        self.update_lever_inner(lever, value);
        self.tick()
    }
    pub fn set_lever(&mut self, lever: LeverHandle) {
        self.update_lever(lever, true)
    }
    pub fn reset_lever(&mut self, lever: LeverHandle) {
        self.update_lever(lever, false)
    }
    pub fn flip_lever(&mut self, lever: LeverHandle) {
        let idx = self.lever_handles[lever.handle];
        self.state.set(idx, !self.state.get_state(idx));
        self.pending_updates.push(idx);
        self.tick();
    }
    pub fn pulse_lever(&mut self, lever: LeverHandle) {
        self.set_lever(lever);
        self.reset_lever(lever);
    }

    pub fn set_lever_stable(&mut self, lever: LeverHandle) {
        self.set_lever(lever);
        self.run_until_stable(10).unwrap();
    }
    pub fn reset_lever_stable(&mut self, lever: LeverHandle) {
        self.reset_lever(lever);
        self.run_until_stable(10).unwrap();
    }
    pub fn flip_lever_stable(&mut self, lever: LeverHandle) {
        self.flip_lever(lever);
        self.run_until_stable(10).unwrap();
    }
    pub fn pulse_lever_stable(&mut self, lever: LeverHandle) {
        self.set_lever(lever);
        self.run_until_stable(10).unwrap();
        self.reset_lever(lever);
        self.run_until_stable(10).unwrap();
    }
    pub(super) fn get_output_handle(&self, handle: CircuitOutputHandle) -> &CircuitOutput {
        &self.output_handles[handle.0]
    }
    pub(super) fn value(&self, idx: GateIndex) -> bool {
        self.state.get_state(idx)
    }
    // Collect only first 8 bits from a larger bus.
    // Or only some bits from a smaller bus.
    pub(super) fn collect_u8_lossy(&self, outputs: &[GateIndex]) -> u8 {
        let mut output = 0;
        let mut mask = 1u8;

        for bit in outputs.iter().take(8) {
            if self.value(*bit) {
                output |= mask
            }

            mask <<= 1;
        }

        output
    }
    // Collect only first 128 bits from a larger bus.
    // Or only some bits from a smaller bus.
    pub(super) fn collect_u128_lossy(&self, outputs: &[GateIndex]) -> u128 {
        let mut output = 0;
        let mut mask = 1u128;

        for bit in outputs.iter().take(128) {
            if self.value(*bit) {
                output |= mask
            }

            mask <<= 1;
        }

        output
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.len() == 0
    }
    pub fn dump_dot(&self, filename: &Path) {
        use petgraph::dot::{Config, Dot};
        use std::io::Write;
        let mut f = std::fs::File::create(filename).unwrap();
        let mut graph = petgraph::Graph::<_, ()>::new();
        let mut index = HashMap::new();
        for (i, node) in self.nodes.iter().enumerate() {
            let is_out = self.outputs.contains(&gi!(i));
            #[cfg(feature = "debug_gates")]
            let name = self
                .names
                .get(&gi!(i))
                .map(|name| format!(":{}", name))
                .unwrap_or("".to_string());

            #[cfg(not(feature = "debug_gates"))]
            let label = if is_out {
                format!("output:{}", node.ty)
            } else {
                node.ty.to_string()
            };
            #[cfg(feature = "debug_gates")]
            let label = if is_out {
                format!("O:{}{}", node.ty, name)
            } else {
                format!("{}{}", node.ty, name)
            };
            index.insert(i, graph.add_node(label));
        }
        for (i, node) in self.nodes.iter().enumerate() {
            graph.extend_with_edges(
                node.dependencies
                    .iter()
                    .map(|dependency| (index[&dependency.idx], index[&i])),
            );
        }
        write!(f, "{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel])).unwrap();
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
