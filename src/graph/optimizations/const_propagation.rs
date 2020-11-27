use super::super::{gate::*, graph_builder::GateGraphBuilder};
use GateType::*;

fn find_replacement(
    g: &mut GateGraphBuilder,
    idx: GateIndex,
    on: bool,
    short_circuit: GateIndex,
    negated: bool,
) -> Option<GateIndex> {
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

    let dependencies_len = g.get(idx).dependencies.len();
    if dependencies_len == 1 {
        return Some(short_circuit_output.opposite_if_const().unwrap());
    }

    let mut non_const_dependency = None;
    for (i, dependency) in g.get(idx).dependencies.iter().copied().enumerate() {
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
                let removed_dep_idx = if i == 0 { 1 } else { 0 };
                let removed_dep = g.get(idx).dependencies[removed_dep_idx];
                g.get_mut(removed_dep).dependents.remove(&idx);

                let gate = g.get_mut(idx);
                gate.ty = Not;
                gate.dependencies.remove(removed_dep_idx);
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
    g: &mut GateGraphBuilder,
    idx: GateIndex,
    on: bool,
    negated: bool,
) -> Option<GateIndex> {
    let dependencies_len = g.get(idx).dependencies.len();
    if dependencies_len == 1 {
        return Some(if negated ^ on { OFF } else { ON });
    }

    let mut non_const_dependency = None;
    let mut output = negated;
    for (i, dependency) in g.get(idx).dependencies.iter().copied().enumerate() {
        if dependency.is_const() {
            output ^= dependency.is_on()
        } else {
            non_const_dependency = Some((dependency, i))
        }
    }
    if let Some((non_const_dependency, i)) = non_const_dependency {
        if dependencies_len == 2 {
            if negated ^ on {
                let removed_dep_idx = if i == 0 { 1 } else { 0 };
                let removed_dep = g.get(idx).dependencies[removed_dep_idx];
                g.get_mut(removed_dep).dependents.remove(&idx);

                let gate = g.get_mut(idx);
                gate.ty = Not;
                gate.dependencies.remove(removed_dep_idx);
                return None;
            } else {
                return Some(non_const_dependency);
            }
        }
        return None;
    }
    Some(if output { ON } else { OFF })
}
// Traverses the graph forwards from constants and nodes with no inputs,
// replacing them with simpler subgraphs.
pub fn const_propagation_pass(g: &mut GateGraphBuilder) {
    // Allocated outside main loop.
    let mut temp_dependents = Vec::new();
    let mut temp_dependencies = Vec::new();

    struct WorkItem {
        idx: GateIndex,
        on: bool,
    }

    // Propagate constants.
    let off = g.get_mut(OFF);

    let mut work: Vec<_> = off
        .dependents
        .drain(0..off.dependents.len())
        .map(|idx| WorkItem { idx, on: false })
        .collect();

    let on = g.get_mut(ON);

    work.extend(
        on.dependents
            .drain(0..on.dependents.len())
            .map(|idx| WorkItem { idx, on: true }),
    );

    for (_, gate) in g.nodes.iter() {
        if !gate.ty.is_lever() && gate.dependencies.is_empty() {
            work.extend(
                gate.dependents
                    .iter()
                    .copied()
                    .map(|idx| WorkItem { idx, on: false }),
            )
        }
    }

    while let Some(WorkItem { idx, on }) = work.pop() {
        if g.nodes.get(idx.into()).is_none() {
            continue;
        }
        if g.is_observable(idx) {
            continue;
        }

        let gate_type = &g.get(idx).ty;
        let replacement = match gate_type {
            Off | On | Lever => unreachable!("Off, On, and lever nodes have no dependencies"),
            Not => Some(if on { OFF } else { ON }),
            And => find_replacement(g, idx, on, OFF, false),
            Nand => find_replacement(g, idx, on, OFF, true),
            Or => find_replacement(g, idx, on, ON, false),
            Nor => find_replacement(g, idx, on, ON, true),
            Xor => find_replacement_xor(g, idx, on, false),
            Xnor => find_replacement_xor(g, idx, on, true),
        };
        if let Some(replacement) = replacement {
            temp_dependents.extend(&g.get(idx).dependents);
            temp_dependencies.extend_from_slice(&g.get(idx).dependencies);

            for dependency in temp_dependencies.drain(0..temp_dependencies.len()) {
                let dependency_dependents = &mut g.get_mut(dependency).dependents;
                dependency_dependents.remove(&idx);
            }

            if replacement.is_const() {
                work.extend(temp_dependents.iter().copied().map(|idx| WorkItem {
                    idx,
                    on: replacement.is_on(),
                }))
            }

            for dependent in temp_dependents.drain(0..temp_dependents.len()) {
                // A gate can have the same dependency many times in different dependency indexes.
                g.get_mut(dependent).swap_dependency(idx, replacement);
                g.nodes
                    .get_mut(replacement.into())
                    .unwrap()
                    .dependents
                    .insert(dependent);
            }

            g.nodes.remove(idx.into());
        }
    }
}
