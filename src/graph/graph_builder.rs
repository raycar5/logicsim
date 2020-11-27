use super::gate::*;
use super::handles::*;
use super::optimizations::*;
use super::InitializedGateGraph;
use crate::data_structures::{Slab, State};
use casey::pascal;
use concat_idents::concat_idents;
use smallvec::smallvec;
use std::collections::{HashMap, HashSet};

use GateType::*;

/// Creates gatename, gatename1, gatename2 and gatenamex constructors for every gate with variable dependencies.
/// The constructors create gates with 0, 1, 2 and x dependencies respectively.
macro_rules! gate_constructors {
    ($name:ident,$($rest:ident),*) => {
        gate_constructors!($name);
        gate_constructors!($($rest),*);
    };
    ($name:ident) => {
        gate_constructors!(
            $name,
            concat!(
                "Returns the [GateIndex] of a new `",
                 stringify!($name),
                  "` gate with no dependencies. Dependencies can be added with [GateGraphBuilder::dpush]\n\n",
                "Providing a good name allows for a great debugging experience, you can disable the \"debug_gates\" feature ",
                "to slightly increase performance"
            ),
            concat!("Returns the [GateIndex] of a new `", stringify!($name), "` gate with 1 dependency."),
            concat!("Returns the [GateIndex] of a new `", stringify!($name), "` gate with 2 dependencies."),
            concat!("Returns the [GateIndex] of a new `", stringify!($name), "` gate with n dependencies.")
        );
    };
    ($name:ident,$doc0:expr,$doc1:expr,$doc2:expr,$docx:expr) => {
        #[doc=$doc0]
        pub fn $name<S: Into<String>>(&mut self, name: S) -> GateIndex {
            let idx = self.nodes.insert(Gate::new(pascal!($name), smallvec![])).into();
            self.create_gate(idx, std::iter::empty(), name);
            idx
        }

        concat_idents!(name1 = $name, 1 {
            // TODO This doesn't work :(
            //#[doc=$doc1]
            /// Returns the [GateIndex] of a new gate with 1 dependency.
            ///
            /// Providing a good name allows for a great debugging experience, you can disable the "debug_gates" feature
            /// to slightly increase performance.
            pub fn name1<S: Into<String>>(&mut self, dep: GateIndex, name: S) -> GateIndex {
                let idx = self.nodes.insert(Gate::new(pascal!($name), smallvec![dep])).into();
                self.create_gate(idx, std::iter::once(dep), name);
                idx
            }
        });

        concat_idents!(name2 = $name, 2 {
            // TODO This doesn't work :(
            //#[doc=$doc2]
            /// Returns the [GateIndex] of a new gate with 2 dependencies.
            ///
            /// Providing a good name allows for a great debugging experience, you can disable the "debug_gates" feature
            /// to slightly increase performance.
            pub fn name2<S: Into<String>>(&mut self, dep1: GateIndex, dep2: GateIndex, name: S) -> GateIndex {
                let idx = self.nodes.insert(Gate::new(pascal!($name), smallvec![dep1, dep2])).into();
                self.create_gate(idx, std::iter::once(dep1).chain(std::iter::once(dep2)), name);
                idx
            }
        });

        concat_idents!(namex = $name, x {
            // TODO This doesn't work :(
            //#[doc=$docx]
            /// Returns the [GateIndex] of a new gate with x dependencies. the dependencies are taken in order from `iter`.
            ///
            /// Providing a good name allows for a great debugging experience, you can disable the "debug_gates" feature
            /// to slightly increase performance.
            pub fn namex<S: Into<String>,I:Iterator<Item=GateIndex>+Clone>(&mut self, iter: I, name: S) -> GateIndex {
                let idx = self.nodes.insert(Gate::new(pascal!($name), iter.clone().collect())).into();
                self.create_gate(idx, iter, name);
                idx
            }
        });
    };
}

/// Data structure that represents a graph of logic gates, it can be [initialized](GateGraphBuilder::init) to simulate the circuit.
///
/// Conceptually the gates are represented as nodes in a graph with dependency edges to other nodes.
///
/// Inputs are represented by constants([ON], [OFF]) and [levers](GateGraphBuilder::lever).
///
/// Outputs are represented by [OutputHandles](OutputHandle) which allow you to query the state of gates and
/// are created by [GateGraphBuilder::output].
///
/// Once the graph is initialized, it transforms into an [InitializedGateGraph] which cannot be modified.
/// The initialization process optimizes the gate graph so that expressive abstractions
/// that potentially generate lots of [constants](GateIndex::is_const) or useless gates can be used without fear.
/// All constants and dead gates will be optimized away and the remaining graph simplified very aggressively.
///
/// **Zero overhead abstractions!**
///
/// # Examples
/// Simple gates.
/// ```
/// # use logicsim::graph::{GateGraphBuilder,ON,OFF};
/// let mut g = GateGraphBuilder::new();
///
/// // Providing each gate with a string name allows for some very neat debugging.
/// // If you don't want them affecting performance, you can disable feature "debug_gates",
/// // all of the strings will be optimized away.
/// let or = g.or2(ON, OFF, "or");
/// let or_output = g.output1(or, "or_output");
///
/// let and = g.and2(ON, OFF, "and");
/// let and_output = g.output1(and, "and_output");
///
/// let ig = &g.init();
///
/// // `b0()` accesses the 0th bit of the output.
/// // Outputs can have as many bits as you want
/// // and be accessed with methods like `u8()`, `char()` or `i128()`.
/// assert_eq!(or_output.b0(ig), true);
/// assert_eq!(and_output.b0(ig), false);
/// ```
///
/// Levers!
/// ```
/// # use logicsim::graph::{GateGraphBuilder,ON,OFF};
/// # let mut g = GateGraphBuilder::new();
/// let l1 = g.lever("l1");
/// let l2 = g.lever("l2");
///
/// let or = g.or2(l1.bit(), l2.bit(), "or");
/// let or_output = g.output1(or, "or_output");
///
/// let and = g.and2(l1.bit(), l2.bit(), "and");
/// let and_output = g.output1(and, "and_output");
///
/// let ig = &mut g.init();
///
/// assert_eq!(or_output.b0(ig), false);
/// assert_eq!(and_output.b0(ig), false);
///
/// // `_stable` means that the graph will run until gate states have stopped changing.
/// // This might not be what you want if you have a circuit that never stabilizes,
/// // like 3 not gates connected in a circle!
/// // See [InitializedGateGraph::run_until_stable].
/// ig.flip_lever_stable(l1);
/// assert_eq!(or_output.b0(ig), true);
/// assert_eq!(and_output.b0(ig), false);
///
/// ig.flip_lever_stable(l2);
/// assert_eq!(or_output.b0(ig), true);
/// assert_eq!(and_output.b0(ig), true);
/// ```
///
/// [SR Latch!](https://en.wikipedia.org/wiki/Flip-flop_(electronics)#SR_NOR_latch)
/// ```
/// # use logicsim::graph::{GateGraphBuilder,ON,OFF};
/// # let mut g = GateGraphBuilder::new();
/// let r = g.lever("l1");
/// let s = g.lever("l2");
///
/// let q = g.nor2(r.bit(), OFF, "q");
/// let nq = g.nor2(s.bit(), q, "nq");
///
/// let q_output = g.output1(q, "q");
/// let nq_output = g.output1(nq, "nq");
///
/// // `d1()` replaces the dependency at index 1 with nq.
/// // We used OFF as a placeholder above.
/// g.d1(q, nq);
///
/// let ig = &mut g.init();
/// // With latches, the initial state should be treated as undefined,
/// // so remember to always reset your latches at the beginning of the simulation.
/// ig.pulse_lever_stable(r);
/// assert_eq!(q_output.b0(ig), false);
/// assert_eq!(nq_output.b0(ig), true);
///
/// ig.pulse_lever_stable(s);
/// assert_eq!(q_output.b0(ig), true);
/// assert_eq!(nq_output.b0(ig), false);
///
/// ig.pulse_lever_stable(r);
/// assert_eq!(q_output.b0(ig), false);
/// assert_eq!(nq_output.b0(ig), true);
/// ```
#[derive(Debug, Clone)]
pub struct GateGraphBuilder {
    pub(super) nodes: Slab<BuildGate>,
    output_handles: Vec<Output>,
    pub(super) lever_handles: Vec<GateIndex>,
    outputs: HashSet<GateIndex>,
    #[cfg(feature = "debug_gates")]
    names: HashMap<GateIndex, String>,
    #[cfg(feature = "debug_gates")]
    probes: HashMap<GateIndex, Probe>,
}
/// Intermediate representation between [GateGraphBuilder] and [InitializedGateGraph].
/// It has the same structure as an [InitializedGateGraph] except for the initialized [State].
///
/// It is only used when transforming a [GateGraphBuilder]
/// into an [InitializedGateGraph] in the [GateGraphBuilder::init] method.
struct CompactedGateGraph {
    nodes: Vec<InitializedGate>,
    output_handles: Vec<Output>,
    lever_handles: Vec<GateIndex>,
    outputs: HashSet<GateIndex>,
    #[cfg(feature = "debug_gates")]
    names: HashMap<GateIndex, String>,
    #[cfg(feature = "debug_gates")]
    probes: HashMap<GateIndex, Probe>,
}

impl GateGraphBuilder {
    /// Returns a new [GateGraphBuilder] containing only [OFF] and [ON].
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

        #[cfg(feature = "debug_gates")]
        let names = {
            let mut names: HashMap<_, _> = Default::default();
            names.insert(OFF, "OFF".into());
            names.insert(ON, "ON".into());
            names
        };

        GateGraphBuilder {
            nodes,
            lever_handles: Default::default(),
            outputs: Default::default(),
            output_handles: Default::default(),
            #[cfg(feature = "debug_gates")]
            names,
            #[cfg(feature = "debug_gates")]
            probes: Default::default(),
        }
    }

    /// Appends `new_dep` to the list of dependencies of gate `target`.
    ///
    /// # Panics
    ///
    /// Will panic if `target` can't have a variable number of dependencies.
    pub fn dpush(&mut self, target: GateIndex, new_dep: GateIndex) {
        let gate = self.get_mut(target.into());
        match gate.ty {
            Off => panic!("OFF has no dependencies"),
            On => panic!("ON has no dependencies"),
            Not => panic!("Not only has one dependency"),
            Lever => panic!("Lever has no dependencies"),
            Or | Nor | And | Nand | Xor | Xnor => {
                gate.dependencies.push(new_dep);
                self.nodes
                    .get_mut(new_dep.into())
                    .unwrap()
                    .dependents
                    .insert(target);
            }
        }
    }

    /// Sets the dependency at index `x` in `target` dependencies to `new_dep`.
    ///
    /// # Panics
    ///
    /// Will panic if `target` has less than `x` + 1 dependencies, you probably want [GateGraphBuilder::dpush] instead.
    ///
    /// Will panic if `target` is Not and `x` > 0.
    ///
    /// Will panic if `target` can't have dependencies.
    pub fn dx(&mut self, target: GateIndex, new_dep: GateIndex, x: usize) {
        let gate = self.nodes.get_mut(target.into()).unwrap();
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
            .get_mut(old_dep.into())
            .unwrap()
            .dependents
            .remove(&target);
        self.nodes
            .get_mut(new_dep.into())
            .unwrap()
            .dependents
            .insert(target);
    }

    /// Sets the dependency at index 0 in `target` dependencies to `new_dep`.
    ///
    /// # Panics
    ///
    /// Will panic if `target` has less than 1 dependency, you probably want [GateGraphBuilder::dpush] instead.
    ///
    /// Will panic if `target` can't have dependencies.
    pub fn d0(&mut self, target: GateIndex, new_dep: GateIndex) {
        self.dx(target, new_dep, 0)
    }

    /// Sets the dependency at index 1 in `target` dependencies to `new_dep`.
    ///
    /// # Panics
    ///
    /// Will panic if `target` has less than 1 dependency, you probably want [GateGraphBuilder::dpush] instead.
    ///
    /// Will panic if `target` can't have more than 1 dependency.
    pub fn d1(&mut self, target: GateIndex, new_dep: GateIndex) {
        self.dx(target, new_dep, 1)
    }

    /// Creates the dependent edges and saves the name of new gates.
    #[allow(unused_variables)]
    fn create_gate<S: Into<String>, I: Iterator<Item = GateIndex>>(
        &mut self,
        idx: GateIndex,
        deps: I,
        name: S,
    ) {
        for dep in deps {
            self.nodes
                .get_mut(dep.into())
                .unwrap()
                .dependents
                .insert(idx);
        }
        #[cfg(feature = "debug_gates")]
        self.names.insert(idx, name.into());
    }

    /// Returns the [LeverHandle] of a new lever gate.
    ///
    /// Providing a good name allows for a great debugging experience.
    /// You can disable the "debug_gates" feature to slightly increase performance.
    pub fn lever<S: Into<String>>(&mut self, name: S) -> LeverHandle {
        let idx = self.nodes.insert(Gate::new(Lever, smallvec![])).into();
        let handle = self.lever_handles.len();
        self.lever_handles.push(idx);
        self.create_gate(idx, std::iter::empty(), name);
        LeverHandle { handle, idx }
    }

    /// Returns the [GateIndex] of a new not gate with 1 dependency.
    ///
    /// Providing a good name allows for a great debugging experience.
    /// You can disable the "debug_gates" feature to slightly increase performance.
    pub fn not1<S: Into<String>>(&mut self, dep: GateIndex, name: S) -> GateIndex {
        let idx = self.nodes.insert(Gate::new(Not, smallvec![dep])).into();
        self.create_gate(idx, std::iter::once(dep), name);
        idx
    }

    // Create constructors for all gate types with variable dependencies.
    gate_constructors!(or, nor, and, nand, xor, xnor);

    /// Returns an immutable reference to the [BuildGate] at `idx`.
    ///
    /// # Panics
    ///
    /// Will panic if `idx` >= self.nodes.len().
    ///
    /// Will panic if `idx` has been removed from self.nodes.
    #[inline(always)]
    pub(super) fn get(&self, idx: GateIndex) -> &BuildGate {
        self.nodes.get(idx.into()).unwrap()
    }

    /// Returns a mutable reference to the [BuildGate] at `idx`.
    ///
    /// # Panics
    ///
    /// Will panic if `idx` >= self.nodes.len().
    ///
    /// Will panic if `idx` has been removed from self.nodes.
    #[inline(always)]
    pub(super) fn get_mut(&mut self, idx: GateIndex) -> &mut BuildGate {
        self.nodes.get_mut(idx.into()).unwrap()
    }

    /// Returns a new [InitializedGateGraph] created from `self` after running optimizations.
    pub fn init(mut self) -> InitializedGateGraph {
        self.optimize();
        self.init_unoptimized()
    }

    /// Returns a new [CompactedGateGraph] created from `self`.
    ///
    /// Compacted means that all gates are placed contiguously and all references to them
    /// are updated accordingly.
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
                nodes: nodes.into_iter().map(|(_, gate)| gate.into()).collect(),
                #[cfg(feature = "debug_gates")]
                names,
                #[cfg(feature = "debug_gates")]
                probes,
                outputs,
                lever_handles,
                output_handles,
            };
        }

        let mut index_map = HashMap::<GateIndex, GateIndex>::new();
        let mut new_nodes = Vec::<InitializedGate>::new();
        index_map.reserve(nodes.len());
        new_nodes.reserve(nodes.len());

        for (new_index, (old_index, gate)) in nodes.into_iter().enumerate() {
            index_map.insert(old_index.into(), gi!(new_index));

            new_nodes.push(gate.into());
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
            .map(|(idx, mut probe)| {
                for bit in &mut probe.bits {
                    *bit = index_map[bit]
                }
                (index_map[&idx], probe)
            })
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

    /// Returns a new [InitializedGateGraph] created from `self` without running optimizations.
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

        let mut state = State::new(nodes.len());
        state.set(OFF.idx, false);
        state.set(ON.idx, true);
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
            if !idx.is_const() && new_graph.state.get_updated(i) {
                continue;
            }
            new_graph.propagation_queue.push(idx);
            new_graph.tick_inner();
        }
        new_graph.pending_updates.swap();
        new_graph
    }

    /// Runs optimization `f` and prints the results of the optimization.
    fn run_optimization<F: Fn(&mut GateGraphBuilder)>(&mut self, f: F, name: &'static str) {
        let old_len = self.len();
        f(self);
        println!(
            "Optimization: {}, old size:{}, new size:{}, reduction: {:.1}%",
            name,
            old_len,
            self.len(),
            (old_len - self.len()) as f32 / old_len as f32 * 100.
        );
    }

    /// Runs all optimizations.
    fn optimize(&mut self) {
        self.run_optimization(const_propagation_pass, "const propagation");
        self.run_optimization(not_deduplication_pass, "not deduplication");
        self.run_optimization(
            single_dependency_collapsing_pass,
            "single dependency collapsing",
        );
        self.run_optimization(dead_code_elimination_pass, "dead code elimination");
        self.run_optimization(global_value_numbering_pass, "global value numbering");
        self.run_optimization(equal_gate_merging_pass, "equal gate merging");
        self.run_optimization(dependency_deduplication_pass, "dependency deduplication");
        self.run_optimization(const_propagation_pass, "const propagation");
    }

    /// Returns true if `gate` is a lever or outputs/probes contain `gate`.
    pub(super) fn is_observable(&self, gate: GateIndex) -> bool {
        if self.outputs.contains(&gate) {
            return true;
        }
        if self.get(gate).ty.is_lever() {
            return true;
        }
        #[cfg(feature = "debug_gates")]
        if self.probes.contains_key(&gate) {
            return true;
        }
        false
    }

    /// Returns a new [OutputHandle] with name `name` for the gates in `bits`.
    ///
    /// See [OutputHandle] for gate querying methods.
    pub fn output<S: Into<String>>(&mut self, bits: &[GateIndex], name: S) -> OutputHandle {
        for bit in bits {
            self.outputs.insert(*bit);
        }
        self.output_handles.push(Output {
            bits: bits.into(),
            name: name.into(),
        });
        OutputHandle(self.output_handles.len() - 1)
    }

    /// Returns a new [OutputHandle] with name `name` for a single gate `bit`.
    ///
    /// See [OutputHandle] for gate querying methods.
    pub fn output1<S: Into<String>>(&mut self, bit: GateIndex, name: S) -> OutputHandle {
        self.output(&[bit], name)
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
    /// OUT:? means if the gate is an output it will be "OUT:" otherwise, it will be "".
    pub(super) fn full_name(&self, gate: GateIndex) -> String {
        let out = if self.outputs.contains(&gate) {
            "OUT:"
        } else {
            ""
        };
        #[cfg(feature = "debug_gates")]
        return format!("{}{}:{}", out, self.get(gate).ty, self.name(gate));
        #[cfg(not(feature = "debug_gates"))]
        format!("{}{}", out, self.get(gate).ty)
    }

    /// Dumps the graph in [dot](https://en.wikipedia.org/wiki/DOT_(graph_description_language)) format
    /// to path `filename`, to be visualized by many supported tools, I recommend [gephi](https://gephi.org/).
    // TODO dry
    pub fn dump_dot(&self, filename: &'static str) {
        use petgraph::dot::{Config, Dot};
        use std::io::Write;
        let mut f = std::fs::File::create(filename).unwrap();
        let mut graph = petgraph::Graph::<_, ()>::new();
        let mut index = HashMap::new();
        for (i, _) in self.nodes.iter() {
            let label = self.full_name(i.into());
            index.insert(i, graph.add_node(label));
        }
        for (i, node) in self.nodes.iter() {
            graph.extend_with_edges(
                node.dependencies
                    .iter()
                    .map(|dependency| (index[&dependency.into()], index[&i])),
            );
        }
        write!(f, "{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel])).unwrap();
    }

    /// "Probes" the gates in `bits`, meaning that whenever the state of any of them changes,
    /// the new state of the group will be printed to stdout along with `name`.
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

    /// "Probes" the gate `bit`, meaning that whenever its state changes,
    /// the new state will be printed to stdout along with `name`.
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

        let set = g.lever("set");
        let reset = g.lever("reset");

        let flip = g.or2(reset.bit(), OFF, "flip");
        let q = g.not1(flip, "q");

        let flop = g.or2(set.bit(), q, "flop");
        let nq = g.not1(flop, "nq");
        g.d1(flip, nq);

        let output = g.output1(nq, "nq");
        let g = &mut graph.init();

        g.run_until_stable(10).unwrap();
        for _ in 0..10 {
            assert_eq!(output.b0(g), true);
        }
        g.update_lever(set, true);

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

        let n1 = g.not1(OFF, "n1");
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
