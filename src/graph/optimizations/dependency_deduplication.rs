use super::super::{graph_builder::GateGraphBuilder, types::*};
use smallvec::SmallVec;
use std::collections::HashMap;
use GateType::*;

// Removes duplicate dependencies from most gates
// If the gate is an Xor or Xnor it keeps 1 if there is an odd number of copies
// or 2 if there is an even number of copies.
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
                g.get_mut(idx).dependencies.push(duplicate)
            }
        }
    }
}
