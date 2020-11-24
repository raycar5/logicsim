use super::optimizations::*;
use super::types::*;
use super::InitializedGateGraph;
use crate::data_structures::{Slab, State};
use crate::gi;
use smallvec::smallvec;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use GateType::*;

#[derive(Debug, Clone)]
pub struct GateGraphBuilder {
    pub(super) nodes: Slab<Gate>,
    output_handles: Vec<CircuitOutput>,
    lever_handles: Vec<GateIndex>,
    outputs: HashSet<GateIndex>,
    #[cfg(feature = "debug_gates")]
    names: HashMap<GateIndex, String>,
    #[cfg(feature = "debug_gates")]
    probes: HashMap<GateIndex, Probe>,
}
struct CompactedGateGraph {
    nodes: Vec<Gate>,
    output_handles: Vec<CircuitOutput>,
    lever_handles: Vec<GateIndex>,
    outputs: HashSet<GateIndex>,
    #[cfg(feature = "debug_gates")]
    names: HashMap<GateIndex, String>,
    #[cfg(feature = "debug_gates")]
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
            #[cfg(feature = "debug_gates")]
            names: Default::default(),
            #[cfg(feature = "debug_gates")]
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
            Or | Nor | And | Nand | Xor | Xnor | Not => {
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
        #[cfg(feature = "debug_gates")]
        self.names.insert(idx, name.into());
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

    fn compacted(self) -> CompactedGateGraph {
        #[cfg(feature = "debug_gates")]
        let GateGraphBuilder {
            names,
            nodes,
            probes,
            outputs,
            output_handles,
            lever_handles,
        } = self;
        #[cfg(not(feature = "debug_gates"))]
        let GateGraphBuilder {
            nodes,
            outputs,
            output_handles,
            lever_handles,
        } = self;
        if nodes.len() == nodes.total_len() {
            return CompactedGateGraph {
                nodes: nodes.into_iter().map(|(_, gate)| gate).collect(),
                #[cfg(feature = "debug_gates")]
                names,
                #[cfg(feature = "debug_gates")]
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

        #[cfg(feature = "debug_gates")]
        let new_names = names
            .into_iter()
            .filter_map(|(idx, name)| Some((*index_map.get(&idx)?, name)))
            .collect();

        #[cfg(feature = "debug_gates")]
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

        CompactedGateGraph {
            #[cfg(feature = "debug_gates")]
            names: new_names,
            nodes: new_nodes,
            #[cfg(feature = "debug_gates")]
            probes: new_probes,
            outputs: new_outputs,
            output_handles: new_output_handles,
            lever_handles: new_lever_handles,
        }
    }
    pub fn init_unoptimized(self) -> InitializedGateGraph {
        #[cfg(feature = "debug_gates")]
        let CompactedGateGraph {
            names,
            nodes,
            probes,
            outputs,
            output_handles,
            lever_handles,
        } = self.compacted();
        #[cfg(not(feature = "debug_gates"))]
        let CompactedGateGraph {
            nodes,
            outputs,
            output_handles,
            lever_handles,
        } = self.compacted();

        let state = State::new(nodes.len());
        let mut new_graph = InitializedGateGraph {
            #[cfg(feature = "debug_gates")]
            names: names.into(),
            nodes: nodes.into(),
            #[cfg(feature = "debug_gates")]
            probes: probes.into(),
            outputs: outputs.into(),
            output_handles: output_handles.into(),
            lever_handles: lever_handles.into(),
            propagation_queue: Default::default(),
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
        new_graph.pending_updates.swap();
        new_graph
    }

    // Optimizations
    fn optimize(&mut self) {
        let old_len = self.len();
        const_propagation_pass(self);
        println!(
            "Optimized const propagation, old size:{}, new size:{}, reduction: {:.1}%",
            old_len,
            self.len(),
            (old_len - self.len()) as f64 / old_len as f64 * 100f64
        );

        let old_len = self.len();
        dead_code_elimination_pass(self);
        println!(
            "Optimized dead code elimination, old size:{}, new size:{}, reduction: {:.1}%",
            old_len,
            self.len(),
            (old_len - self.len()) as f64 / old_len as f64 * 100f64
        );

        let old_len = self.len();
        duplicate_dependency_elimination_pass(self);
        println!(
            "Optimized duplicate dependency, old size:{}, new size:{}, reduction: {:.1}%",
            old_len,
            self.len(),
            (old_len - self.len()) as f64 / old_len as f64 * 100f64
        );

        let old_len = self.len();
        const_propagation_pass(self);
        println!(
            "Optimized const propagation, old size:{}, new size:{}, reduction: {:.1}%",
            old_len,
            self.len(),
            (old_len - self.len()) as f64 / old_len as f64 * 100f64
        );
    }

    // Output operations.
    pub(super) fn is_observable(&self, gate: GateIndex) -> bool {
        if self.outputs.contains(&gate) {
            return true;
        }
        #[cfg(feature = "debug_gates")]
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
    #[cfg(feature = "debug_gates")]
    pub fn probe<S: Into<String>>(&mut self, bits: &[GateIndex], name: S) {
        let name = name.into();
        for bit in bits {
            self.probes.insert(
                *bit,
                Probe {
                    name: name.clone(),
                    bits: smallvec::SmallVec::from_slice(bits),
                },
            );
        }
    }
    #[cfg(feature = "debug_gates")]
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
