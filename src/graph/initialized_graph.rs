use super::gate::*;
use super::handles::*;
use crate::data_structures::{DoubleStack, Immutable, State};
use concat_idents::concat_idents;
use std::collections::{HashMap, HashSet};

macro_rules! type_collectors {
    ($ty:ident,$($rest:ident),*) => {
        type_collectors!($ty);
        type_collectors!($($rest),*);
    };
    ($ty:ident) => {
        concat_idents!(collect_t = collect, _, $ty, _, lossy {
            pub fn collect_t(&self, outputs: &[GateIndex]) -> $ty {
                let mut output = 0;
                let mut mask = 1;

                for bit in outputs.iter().take(std::mem::size_of::<$ty>()*8) {
                    if self.value(*bit) {
                        output |= mask
                    }

                    mask <<= 1;
                }

                output
            }
        });
    };
}
pub struct InitializedGateGraph {
    // Making node immutable makes the program slightly slower when the binary includes debug information.
    pub(super) nodes: Immutable<Vec<InitializedGate>>,
    pub(super) pending_updates: DoubleStack<GateIndex>,
    pub(super) propagation_queue: DoubleStack<GateIndex>, // Allocated outside to prevent allocations in the hot loop.
    pub(super) output_handles: Immutable<Vec<Output>>,
    pub(super) lever_handles: Immutable<Vec<GateIndex>>,
    pub(super) outputs: Immutable<HashSet<GateIndex>>,
    pub(super) state: State,
    #[cfg(feature = "debug_gates")]
    pub(super) names: Immutable<HashMap<GateIndex, String>>,
    #[cfg(feature = "debug_gates")]
    pub(super) probes: Immutable<HashMap<GateIndex, Probe>>,
}
use GateType::*;
impl InitializedGateGraph {
    #[inline(always)]
    fn fold_short(&self, ty: &GateType, gates: &[GateIndex]) -> bool {
        let init = ty.init();
        let short = !init;
        // Using a manual loop results in 2% less instructions.
        #[allow(clippy::needless_range_loop)]
        for i in 0..gates.len() {
            // This is safe because in an InitializedGraph nodes.len() <= state.len().
            let state = unsafe { self.state.get_state_very_unsafely(gates[i].idx) };
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
        // Check the State unsafe invariant once instead of on every call.
        debug_assert!(self.nodes.len() <= self.state.len());
        while !self.propagation_queue.is_empty() {
            self.propagation_queue.swap();
            while let Some(idx) = self.propagation_queue.pop() {
                // This is safe because the propagation queue gets filled by items coming from
                // nodes.iter() or levers, both of which are always in bounds.
                debug_assert!(idx.idx < self.nodes.len());
                let node = unsafe { self.nodes.get_unchecked(idx.idx) };

                let new_state = match &node.ty {
                    On => true,
                    Off => false,
                    // This is safe because in an InitializedGraph nodes.len() <= state.len().
                    Lever => unsafe { self.state.get_state_very_unsafely(idx.idx) },
                    Not => unsafe { !self.state.get_state_very_unsafely(node.dependencies[0].idx) },
                    Or | Nor | And | Nand | Xor | Xnor => {
                        let mut new_state = if node.ty.short_circuits() {
                            self.fold_short(&node.ty, &node.dependencies)
                        } else {
                            let mut result = node.ty.init();

                            // Using a manual loop results in 2% less instructions.
                            #[allow(clippy::needless_range_loop)]
                            for i in 0..node.dependencies.len() {
                                // This is safe because in an InitializedGraph nodes.len() <= state.len().
                                let state = unsafe {
                                    self.state.get_state_very_unsafely(node.dependencies[i].idx)
                                };
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
                // This is safe because in an InitializedGraph nodes.len() <= state.len().
                let old_state = unsafe { self.state.get_state_very_unsafely(idx.idx) };

                // This is safe because in an InitializedGraph nodes.len() <= state.len().
                if unsafe { self.state.get_updated_very_unsafely(idx.idx) } {
                    if old_state != new_state {
                        self.pending_updates.push(idx);
                    }
                    continue;
                }
                unsafe { self.state.set_very_unsafely(idx.idx, new_state) };

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
                    self.propagation_queue.extend_from_slice(&node.dependents)
                }
            }
        }
    }
    pub fn tick(&mut self) {
        while let Some(pending) = &self.pending_updates.pop() {
            self.state.tick();
            self.propagation_queue.push(*pending);
            self.tick_inner()
        }
        self.pending_updates.swap();
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
        if self.state.get_state(idx.idx) != value {
            self.state.set(idx.idx, value);
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
        self.state.set(idx.idx, !self.state.get_state(idx.idx));
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
    pub(super) fn get_output_handle(&self, handle: OutputHandle) -> &Output {
        &self.output_handles[handle.0]
    }
    pub(super) fn value(&self, idx: GateIndex) -> bool {
        self.state.get_state(idx.idx)
    }

    type_collectors!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

    pub fn collect_char_lossy(&self, outputs: &[GateIndex]) -> char {
        self.collect_u8_lossy(outputs) as char
    }

    /// Returns the number of gates in the graph.
    // The graph always contains OFF and ON.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the name of `gate`.
    #[cfg(feature = "debug_gates")]
    pub(super) fn name(&self, gate: GateIndex) -> &str {
        &self.names[&gate]
    }

    /// Returns the "full name" of `gate` in format:
    ///
    /// "OUT:?GATE_TYPE:GATE_NAME" if the "debug_gates" feature is enabled.
    ///
    /// "OUT:?GATE_TYPE" if the "debug_gates" feature is disabled.
    ///
    /// OUT:? means if the gate is an output it will be "OUT:" and "" otherwise.
    pub(super) fn full_name(&self, gate: GateIndex) -> String {
        let out = if self.outputs.contains(&gate) {
            "OUT:"
        } else {
            ""
        };
        #[cfg(feature = "debug_gates")]
        return format!("{}{}:{}", out, self.nodes[gate.idx].ty, self.name(gate));
        #[cfg(not(feature = "debug_gates"))]
        format!("{}{}", out, self.nodes[gate.idx].ty)
    }

    /// Dumps the graph in [dot](https://en.wikipedia.org/wiki/DOT_(graph_description_language)) format
    /// to path `filename`, to be visualized by many supported tools, I recommend [gephi](https://gephi.org/).
    pub fn dump_dot(&self, filename: &'static str) {
        use petgraph::dot::{Config, Dot};
        use std::io::Write;
        let mut f = std::fs::File::create(filename).unwrap();
        let mut graph = petgraph::Graph::<_, ()>::new();
        let mut index = HashMap::new();
        for (i, _) in self.nodes.iter().enumerate() {
            let label = self.full_name(gi!(i));
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
