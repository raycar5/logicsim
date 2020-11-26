use super::super::{graph_builder::GateGraphBuilder, types::*};

// Replaces all not gates coming from the same gate with a single one.
pub fn not_deduplication_pass(g: &mut GateGraphBuilder) {
    let mut nots = Vec::new();
    let mut temp_deps = Vec::new();
    struct WorkItem {
        gate: GateIndex,
        first_not: GateIndex,
    }
    let mut work: Vec<WorkItem> = g
        .nodes
        .iter()
        .filter_map(|(i, gate)| {
            let mut first_not = None;
            for dependent in &gate.dependents {
                let dependent_gate = g.get(*dependent);
                match (first_not, dependent_gate.ty.is_not()) {
                    (Some(first_not), true) => {
                        return WorkItem {
                            gate: i.into(),
                            first_not,
                        }
                        .into()
                    }
                    (None, true) => first_not = Some(*dependent),
                    _ => {}
                }
            }
            None
        })
        .collect();

    while let Some(WorkItem { gate, first_not }) = work.pop() {
        nots.extend(
            g.nodes
                .get(gate.into())
                .unwrap()
                .dependents
                .iter()
                .copied()
                .filter(|dependent| {
                    *dependent != first_not
                        && *dependent != gate
                        && !g.is_observable(*dependent)
                        && g.get(*dependent).ty.is_not()
                }),
        );
        for not in nots.drain(0..nots.len()) {
            temp_deps.extend(g.get(not).dependents.iter().copied());

            for dependent in temp_deps.drain(0..temp_deps.len()) {
                for dependent_dependency in &mut g.get_mut(dependent).dependencies {
                    if *dependent_dependency == not {
                        *dependent_dependency = first_not
                    }
                }
                g.nodes
                    .get_mut(first_not.into())
                    .unwrap()
                    .dependents
                    .insert(dependent);
            }

            g.nodes.remove(not.into());
            g.get_mut(gate).dependents.remove(&not);
        }
    }
}
