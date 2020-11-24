use super::super::{graph_builder::GateGraphBuilder, types::*};
// Traverses the graph backwards removing all nodes with no dependents.
pub fn dead_code_elimination_pass(g: &mut GateGraphBuilder) {
    let mut temp_dependencies = Vec::new();

    let mut work: Vec<_> = g
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
        if g.is_observable(idx) {
            continue;
        }
        temp_dependencies.extend_from_slice(&g.nodes.get(idx.idx).unwrap().dependencies);

        for dependency in temp_dependencies.drain(0..temp_dependencies.len()) {
            let dependency_gate = g.nodes.get_mut(dependency.idx).unwrap();
            dependency_gate.dependents.remove(&idx);
            if dependency_gate.dependents.is_empty() {
                work.push(dependency)
            }
        }
        g.nodes.remove(idx.idx);
    }
}
