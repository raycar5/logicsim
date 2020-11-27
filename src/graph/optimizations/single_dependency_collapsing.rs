use super::super::{gate::*, graph_builder::GateGraphBuilder};

use smallvec::SmallVec;
use GateType::*;
// Simplifies nodes with a single dependency to negated copies or the dependency itself.
pub fn single_dependency_collapsing_pass(g: &mut GateGraphBuilder) {
    let mut temp_dep_deps = SmallVec::<[GateIndex; GATE_DEPENDENCIES_TINYVEC_SIZE]>::new();
    let mut work: Vec<_> = g
        .nodes
        .iter()
        .filter_map(|(i, gate)| {
            let idx = i.into();
            if gate.dependencies.len() == 1
                && gate.dependencies[0] != idx
                && !g.get(gate.dependencies[0]).dependencies.contains(&idx)
            {
                Some(idx)
            } else {
                None
            }
        })
        .collect();

    while let Some(idx) = work.pop() {
        if g.is_observable(idx) {
            continue;
        }
        // It could have been modified by a previous operation
        if g.get(idx).dependencies.len() != 1 {
            continue;
        }
        let ty = g.get(idx).ty;
        let dependency = g.get(idx).dependencies[0];
        match ty {
            Off | On | Lever => unreachable!("Off, On, and lever nodes have no dependencies"),
            Not | Nand | Nor | Xnor => {
                if !g.get(dependency).ty.has_negated_version() {
                    g.get_mut(idx).ty = Not;
                    continue;
                }
                // if the dependency has only one dependent (idx) then we can move idx.dependents to
                // the dependency and negate it.
                if g.get(dependency).dependents.len() == 1 {
                    let dependents = std::mem::take(&mut g.get_mut(idx).dependents);
                    g.get_mut(dependency).dependents.remove(&idx);
                    for dependant in dependents {
                        g.get_mut(dependency).dependents.insert(dependant);
                        g.get_mut(dependant).swap_dependency(idx, dependency);
                    }
                    g.get_mut(dependency).ty = g.get(dependency).ty.negated_version();
                    g.nodes.remove(idx.into());
                // if it has more than one dependent then idx can become the negated version of dependency;
                } else {
                    let dep_gate = g.get(dependency);
                    let dep_type = dep_gate.ty;
                    temp_dep_deps.extend_from_slice(&dep_gate.dependencies);
                    for dep_dep in &temp_dep_deps {
                        g.get_mut(*dep_dep).dependents.insert(idx);
                    }
                    g.get_mut(idx).dependencies = temp_dep_deps.clone();
                    temp_dep_deps.clear();
                    g.get_mut(idx).ty = dep_type.negated_version();
                    g.get_mut(dependency).dependents.remove(&idx);
                }
            }
            And | Or | Xor => {
                let dependents = std::mem::take(&mut g.get_mut(idx).dependents);
                g.get_mut(dependency).dependents.remove(&idx);
                for dependant in dependents {
                    g.get_mut(dependant).swap_dependency(idx, dependency);
                    g.get_mut(dependency).dependents.insert(dependant);
                }
                g.nodes.remove(idx.into());
            }
        }
    }
}
