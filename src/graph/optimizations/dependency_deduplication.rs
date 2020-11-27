use super::super::{gate::*, graph_builder::GateGraphBuilder};
use smallvec::SmallVec;
use std::collections::HashMap;
use GateType::*;

/// Removes duplicate dependencies from gates.
/// If the gate is an Xor or Xnor it keeps 1 if there are an odd number of copies
/// or 2 if there are an even number of copies.
pub fn dependency_deduplication_pass(g: &mut GateGraphBuilder) {
    struct WorkItem {
        idx: GateIndex,
        duplicates: SmallVec<[(GateIndex, usize); 2]>,
    }

    let mut work: Vec<WorkItem> = g
        .nodes
        .iter()
        .filter_map(|(idx, gate)| {
            let mut dependency_multi_map = HashMap::<GateIndex, usize>::new();
            // Detect duplicate dependencies and how many times they are duplicated.
            for dependency in gate.dependencies.iter().copied() {
                let entry = dependency_multi_map.entry(dependency).or_default();
                *entry += 1
            }

            let idx = idx.into();
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
    use Action::*;
    while let Some(WorkItem { idx, duplicates }) = work.pop() {
        let gate_dependencies = &mut g.get_mut(idx).dependencies;
        gate_dependencies.sort();
        gate_dependencies.dedup();
        for (duplicate, count) in duplicates {
            let gate_type = &g.get(idx).ty;
            let action = match gate_type {
                Off | On | Lever => {
                    unreachable!("Off, On, and lever nodes have no dependencies")
                }
                Not => unreachable!("Not gates only have 1 dependency"),

                And | Nand | Or | Nor => Keep1,
                Xor | Xnor => {
                    if count % 2 == 0 {
                        Keep2
                    } else {
                        Keep1
                    }
                }
            };
            if let Keep2 = action {
                g.get_mut(idx).dependencies.push(duplicate)
            }
        }
    }
}
