use super::super::{graph_builder::GateGraphBuilder, types::*};
use smallvec::SmallVec;
use GateType::*;

fn find_replacement(
    g: &mut GateGraphBuilder,
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
    let dependencies_len = g.nodes.get(idx_usize).unwrap().dependencies.len();
    if dependencies_len == 1 {
        if from_const {
            return Some(short_circuit_output.opposite_if_const().unwrap());
        }
        if negated {
            g.nodes.get_mut(idx_usize).unwrap().ty = Not;
            return None;
        }
        return Some(g.nodes.get(idx_usize).unwrap().dependencies[0]);
    }

    let mut non_const_dependency = None;
    for (i, dependency) in g
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
                let gate = g.nodes.get_mut(idx_usize).unwrap();
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
    g: &mut GateGraphBuilder,
    idx: GateIndex,
    on: bool,
    from_const: bool,
    negated: bool,
) -> Option<GateIndex> {
    let idx_usize = idx.idx;
    let dependencies_len = g.nodes.get(idx_usize).unwrap().dependencies.len();
    if dependencies_len == 1 {
        if from_const {
            return Some(if negated ^ on { OFF } else { ON });
        }
        if negated ^ on {
            g.nodes.get_mut(idx_usize).unwrap().ty = Not;
            return None;
        }
        return Some(g.nodes.get(idx_usize).unwrap().dependencies[0]);
    }

    let mut non_const_dependency = None;
    let mut output = negated;
    for (i, dependency) in g
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
                let gate = g.nodes.get_mut(idx_usize).unwrap();
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
pub fn const_propagation_pass(g: &mut GateGraphBuilder) {
    // Allocated outside main loop.
    let mut temp_dependents = Vec::new();
    let mut temp_dependencies = Vec::new();

    struct WorkItem {
        idx: GateIndex,
        on: bool,
        from_const: bool,
    }

    // Propagate constants.
    let off = g.nodes.get_mut(OFF.idx).unwrap();

    let mut work: Vec<_> = off
        .dependents
        .drain(0..off.dependents.len())
        .map(|idx| WorkItem {
            idx,
            on: false,
            from_const: true,
        })
        .collect();

    let on = g.nodes.get_mut(ON.idx).unwrap();

    work.extend(
        on.dependents
            .drain(0..on.dependents.len())
            .map(|idx| WorkItem {
                idx,
                on: true,
                from_const: true,
            }),
    );

    work.extend(g.nodes.iter().filter_map(|(idx, gate)| {
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
        if g.is_observable(idx) {
            continue;
        }
        if g.nodes.get(idx.idx).is_none() {
            continue;
        }

        let gate_type = &g.nodes.get(idx.idx).unwrap().ty;
        let replacement = match gate_type {
            Off | On | Lever => unreachable!("Off, On, and lever nodes have no dependencies"),
            Not => {
                if from_const {
                    Some(if on { OFF } else { ON })
                } else {
                    None
                }
            }
            And => find_replacement(g, idx, on, from_const, OFF, false),
            Nand => find_replacement(g, idx, on, from_const, OFF, true),
            Or => find_replacement(g, idx, on, from_const, ON, false),
            Nor => find_replacement(g, idx, on, from_const, ON, true),
            Xor => find_replacement_xor(g, idx, on, from_const, false),
            Xnor => find_replacement_xor(g, idx, on, from_const, true),
        };
        if let Some(replacement) = replacement {
            temp_dependents.extend(&g.nodes.get(idx.idx).unwrap().dependents);
            temp_dependencies.extend_from_slice(&g.nodes.get(idx.idx).unwrap().dependencies);

            for dependency in temp_dependencies.drain(0..temp_dependencies.len()) {
                let dependency_dependents =
                    &mut g.nodes.get_mut(dependency.idx).unwrap().dependents;
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
                let positions = g
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
                    g.nodes.get_mut(dependent.idx).unwrap().dependencies[position] = replacement
                }
                g.nodes
                    .get_mut(replacement.idx)
                    .unwrap()
                    .dependents
                    .insert(dependent);
            }

            g.nodes.remove(idx.idx);
        }
    }
}
