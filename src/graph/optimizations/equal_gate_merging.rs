use super::{
    super::{graph_builder::GateGraphBuilder, types::*},
    dead_code_elimination_pass,
};

// Merges Gates of the same type
pub fn equal_gate_merging_pass(g: &mut GateGraphBuilder) {
    let mut work: Vec<GateIndex> = g.nodes.iter().map(|(i, _)| gi!(i)).collect();
    let mut temp_deps = Vec::new();
    let mut temp_deps_deps = Vec::new();

    while let Some(idx) = work.pop() {
        let gate = g.get(idx);
        let gate_ty = gate.ty;
        if gate_ty.is_negated() {
            continue;
        }

        temp_deps.extend_from_slice(&gate.dependencies);
        let mut updated = false;

        'dep: for (position, dependency) in temp_deps.drain(0..temp_deps.len()).enumerate().rev() {
            if dependency == idx {
                continue;
            }
            let dependency_gate = g.get(dependency);
            if gate_ty == dependency_gate.ty {
                temp_deps_deps.extend_from_slice(&dependency_gate.dependencies);
                for dep_dep in &temp_deps_deps {
                    if *dep_dep == dependency || *dep_dep == idx {
                        temp_deps_deps.clear();
                        continue 'dep;
                    }
                }
                g.get_mut(idx).dependencies.remove(position);
                for dep_dep in temp_deps_deps.drain(0..temp_deps_deps.len()) {
                    g.get_mut(dep_dep).dependents.insert(idx);
                    g.get_mut(idx).dependencies.push(dep_dep)
                }

                g.get_mut(dependency).dependents.remove(&idx);
                updated = true;
            }
        }
        if updated {
            work.push(idx)
        }
    }
    dead_code_elimination_pass(g);
}
