use super::types::*;
use super::InitializedGateGraph;
use crate::data_structures::{Slab, State};
use crate::gi;
use smallvec::{smallvec, SmallVec};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use GateType::*;

#[derive(Debug, Clone)]
pub struct GateGraphBuilder {
    nodes: Slab<Gate>,
    output_handles: Vec<CircuitOutput>,
    lever_handles: Vec<GateIndex>,
    outputs: HashSet<GateIndex>,
    #[cfg(feature = "debug_gate_names")]
    names: HashMap<GateIndex, String>,
    #[cfg(feature = "debug_gate_names")]
    probes: HashMap<GateIndex, Probe>,
}
struct IntermediateGateGraph {
    nodes: Vec<Gate>,
    output_handles: Vec<CircuitOutput>,
    lever_handles: Vec<GateIndex>,
    outputs: HashSet<GateIndex>,
    #[cfg(feature = "debug_gate_names")]
    names: HashMap<GateIndex, String>,
    #[cfg(feature = "debug_gate_names")]
    probes: HashMap<GateIndex, Probe>,
}
impl GateGraphBuilder {
    pub fn new() -> GateGraphBuilder {
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
        GateGraphBuilder {
            nodes,
            lever_handles: Default::default(),
            outputs: Default::default(),
            output_handles: Default::default(),
            #[cfg(feature = "debug_gate_names")]
            names: Default::default(),
            #[cfg(feature = "debug_gate_names")]
            probes: Default::default(),
        }
    }

    // Dependency operations.
    pub fn dpush(&mut self, idx: GateIndex, new_dep: GateIndex) {
        let gate = self.nodes.get_mut(idx.idx).unwrap();
        match gate.ty {
            Off => panic!("OFF has no dependencies"),
            On => panic!("ON has no dependencies"),
            Lever => panic!("Lever has no dependencies"),
            Or | Nor | And | Nand | Xor | Xnor | Not | Lut(..) => {
                gate.dependencies.push(new_dep);
                self.nodes
                    .get_mut(new_dep.idx)
                    .unwrap()
                    .dependents
                    .insert(idx);
            }
        }
    }
    pub fn dx(&mut self, idx: GateIndex, new_dep: GateIndex, x: usize) {
        let gate = self.nodes.get_mut(idx.idx).unwrap();
        match gate.ty {
            Off => panic!("OFF has no dependencies"),
            On => panic!("ON has no dependencies"),
            Lever => panic!("Lever has no dependencies"),
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
            .remove(&idx);
        self.nodes
            .get_mut(new_dep.idx)
            .unwrap()
            .dependents
            .insert(idx);
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
            self.nodes.get_mut(dep.idx).unwrap().dependents.insert(idx);
        }
        #[cfg(feature = "debug_gate_names")]
        self.names.insert(idx, name.into());
    }
    pub(super) fn gate<S: Into<String>>(&mut self, ty: GateType, name: S) -> GateIndex {
        let idx = gi!(self.nodes.insert(Gate::new(ty, smallvec![])));
        self.create_gate(idx, &[], name);
        idx
    }
    pub fn lever<S: Into<String>>(&mut self, name: S) -> LeverHandle {
        let idx = GateIndex::new(self.nodes.insert(Gate::new(Lever, smallvec![])));
        let handle = self.lever_handles.len();
        self.lever_handles.push(idx);
        self.create_gate(idx, &[], name);
        LeverHandle { handle, idx }
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

    pub fn init(mut self) -> InitializedGateGraph {
        self.optimize();
        self.init_unoptimized()
    }

    fn compacted(self) -> IntermediateGateGraph {
        #[cfg(feature = "debug_gate_names")]
        let GateGraphBuilder {
            names,
            nodes,
            probes,
            outputs,
            output_handles,
            lever_handles,
        } = self;
        let GateGraphBuilder {
            nodes,
            outputs,
            output_handles,
            lever_handles,
        } = self;
        if nodes.len() == nodes.total_len() {
            return IntermediateGateGraph {
                nodes: nodes.into_iter().map(|(_, gate)| gate).collect(),
                #[cfg(feature = "debug_gate_names")]
                names,
                #[cfg(feature = "debug_gate_names")]
                probes,
                outputs,
                lever_handles,
                output_handles,
            };
        }

        let mut index_map = HashMap::new();
        let mut new_nodes = Vec::new();
        index_map.reserve(nodes.len());
        new_nodes.reserve(nodes.len());

        for (new_index, (old_index, gate)) in nodes.into_iter().enumerate() {
            index_map.insert(gi!(old_index), gi!(new_index));
            new_nodes.push(gate);
        }
        for gate in &mut new_nodes {
            for dependency in &mut gate.dependencies {
                *dependency = index_map[dependency];
            }
            gate.dependents = gate.dependents.iter().map(|idx| index_map[idx]).collect();
        }

        #[cfg(feature = "debug_gate_names")]
        let new_names = names
            .into_iter()
            .filter_map(|(idx, name)| Some((*index_map.get(&idx)?, name)))
            .collect();

        #[cfg(feature = "debug_gate_names")]
        let new_probes = probes
            .into_iter()
            .map(|(idx, probe)| (index_map[&idx], probe))
            .collect();

        let new_output_handles = output_handles
            .into_iter()
            .map(|mut output| {
                for bit in &mut output.bits {
                    *bit = index_map[bit]
                }
                output
            })
            .collect();

        let new_lever_handles = lever_handles
            .into_iter()
            .map(|idx| index_map[&idx])
            .collect();

        let new_outputs = outputs.into_iter().map(|idx| index_map[&idx]).collect();

        IntermediateGateGraph {
            #[cfg(feature = "debug_gate_names")]
            names: new_names,
            nodes: new_nodes,
            #[cfg(feature = "debug_gate_names")]
            probes: new_probes,
            outputs: new_outputs,
            output_handles: new_output_handles,
            lever_handles: new_lever_handles,
        }
    }
    pub fn init_unoptimized(self) -> InitializedGateGraph {
        #[cfg(feature = "debug_gate_names")]
        let IntermediateGateGraph {
            names,
            nodes,
            probes,
            outputs,
            output_handles,
            lever_handles,
        } = self.compacted();
        let IntermediateGateGraph {
            nodes,
            outputs,
            output_handles,
            lever_handles,
        } = self.compacted();

        let state = State::new(nodes.len());
        let mut new_graph = InitializedGateGraph {
            #[cfg(feature = "debug_gate_names")]
            names,
            nodes,
            #[cfg(feature = "debug_gate_names")]
            probes,
            outputs,
            output_handles,
            lever_handles,
            propagation_queue: Default::default(),
            next_pending_updates: Default::default(),
            pending_updates: Default::default(),
            state,
        };

        for i in 0..new_graph.len() {
            let idx = gi!(i);
            if idx != OFF && idx != ON && new_graph.state.get_updated(idx) {
                continue;
            }
            new_graph.propagation_queue.push_back(idx);
            new_graph.tick_inner();
        }
        new_graph.pending_updates.extend(
            new_graph
                .next_pending_updates
                .drain(0..new_graph.next_pending_updates.len()),
        );
        new_graph
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
    }
    fn find_replacement(
        &mut self,
        idx: GateIndex,
        on: bool,
        from_const: bool,
        short_circuit: GateIndex,
        negated: bool,
    ) -> Option<GateIndex> {
        let idx_usize = idx.idx;
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
        let dependencies_len = self.nodes.get(idx_usize).unwrap().dependencies.len();
        if dependencies_len == 1 {
            if from_const {
                return Some(short_circuit_output.opposite_if_const().unwrap());
            }
            if negated {
                self.nodes.get_mut(idx_usize).unwrap().ty = Not;
                return None;
            }
            return Some(self.nodes.get(idx_usize).unwrap().dependencies[0]);
        }

        let mut non_const_dependency = None;
        for (i, dependency) in self
            .nodes
            .get(idx_usize)
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
                    let gate = self.nodes.get_mut(idx_usize).unwrap();
                    gate.ty = Not;
                    gate.dependencies.remove(i + 1 % 2);
                    return None;
                } else {
                    return Some(non_const_dependency);
                }
            }
            None
        } else {
            // If there are only const dependencies and none of them are short circuits
            // the output must be the opposite of the short_circuit output.
            Some(short_circuit_output.opposite_if_const().unwrap())
        }
    }
    fn find_replacement_xor(
        &mut self,
        idx: GateIndex,
        on: bool,
        from_const: bool,
        negated: bool,
    ) -> Option<GateIndex> {
        let idx_usize = idx.idx;
        let dependencies_len = self.nodes.get(idx_usize).unwrap().dependencies.len();
        if dependencies_len == 1 {
            if from_const {
                return Some(if negated ^ on { OFF } else { ON });
            }
            if negated ^ on {
                self.nodes.get_mut(idx_usize).unwrap().ty = Not;
                return None;
            }
            return Some(self.nodes.get(idx_usize).unwrap().dependencies[0]);
        }

        let mut non_const_dependency = None;
        let mut output = negated;
        for (i, dependency) in self
            .nodes
            .get(idx_usize)
            .unwrap()
            .dependencies
            .iter()
            .copied()
            .enumerate()
        {
            if dependency.is_const() {
                output ^= dependency.is_on()
            } else {
                non_const_dependency = Some((dependency, i))
            }
        }
        if let Some((non_const_dependency, i)) = non_const_dependency {
            if dependencies_len == 2 {
                if negated ^ on {
                    let gate = self.nodes.get_mut(idx_usize).unwrap();
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
            idx: GateIndex,
            on: bool,
            from_const: bool,
        }

        // Propagate constants.
        let off = self.nodes.get_mut(OFF.idx).unwrap();

        let mut work: Vec<_> = off
            .dependents
            .drain(0..off.dependents.len())
            .map(|idx| WorkItem {
                idx: idx,
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
                    idx: gi!(idx),
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
            if self.is_observable(idx) {
                continue;
            }
            if self.nodes.get(idx.idx).is_none() {
                continue;
            }

            let gate_type = &self.nodes.get(idx.idx).unwrap().ty;
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
                temp_dependents.extend(self.nodes.get(idx.idx).unwrap().dependents.iter());
                temp_dependencies.extend(
                    self.nodes
                        .get(idx.idx)
                        .unwrap()
                        .dependencies
                        .iter()
                        .copied(),
                );

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
                        .get(dependent.idx)
                        .unwrap()
                        .dependencies
                        .iter()
                        .enumerate()
                        .fold(
                            SmallVec::<[usize; 2]>::new(),
                            |mut acc, (position, index)| {
                                if *index == idx {
                                    acc.push(position)
                                }
                                acc
                            },
                        );
                    for position in positions {
                        self.nodes.get_mut(dependent.idx).unwrap().dependencies[position] =
                            replacement
                    }
                    self.nodes
                        .get_mut(replacement.idx)
                        .unwrap()
                        .dependents
                        .insert(dependent);
                }

                self.nodes.remove(idx.idx);
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
                dependency_gate.dependents.remove(&idx);
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
                    *entry += 1
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
    /*
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

            let subgraph = &mut GateGraphBuilder::new();
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
    */

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
    pub fn output<S: Into<String>>(&mut self, bits: &[GateIndex], name: S) -> CircuitOutputHandle {
        for bit in bits {
            self.outputs.insert(*bit);
        }
        self.output_handles.push(CircuitOutput {
            bits: bits.into(),
            name: name.into(),
        });
        CircuitOutputHandle(self.output_handles.len() - 1)
    }
    pub fn output1<S: Into<String>>(&mut self, bit: GateIndex, name: S) -> CircuitOutputHandle {
        self.output(&[bit], name)
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
}

impl Default for GateGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flip_flop() {
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;

        let set = g.lever("");
        let reset = g.lever("");

        let flip = g.or2(reset.bit(), OFF, "");
        let q = g.not1(flip, "");

        let flop = g.or2(set.bit(), q, "");
        let nq = g.not1(flop, "");
        g.d1(flip, nq);

        let output = g.output1(nq, "nq");
        let g = &mut graph.init();

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
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;
        let n1 = g.not("n1");
        let n2 = g.not1(n1, "n2");
        let n3 = g.not1(n2, "n3");
        g.d0(n1, n3);

        let output = g.output1(n1, "n1");
        let g = &mut graph.init();

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
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;
        let and = g.and2(ON, ON, "and");
        let output = g.output(&[and], "big_and");
        g.dpush(and, ON);
        g.dpush(and, ON);
        let g = &mut graph.init();

        assert_eq!(output.b0(g), true)
    }
}
