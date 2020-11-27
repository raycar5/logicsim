use super::super::{gate::*, graph_builder::GateGraphBuilder};
use super::dead_code_elimination_pass;
use std::collections::{hash_map::DefaultHasher, HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct ValueNumber(GateIndex);
type NumberMap = HashMap<GateIndex, ValueNumber>;
type Expression = u64;
fn lookup<I: Iterator<Item = ValueNumber>>(
    op: GateType,
    op_hash_offset: u64,
    dep_nums: I,
    x: GateIndex,
    hash_table: &mut HashMap<Expression, GateIndex>,
) -> GateIndex {
    let op_hash = if op.is_lever() || x.is_const() {
        x.idx as u64
    } else {
        op as u64 + op_hash_offset
    };

    let mut hasher = DefaultHasher::new();
    hasher.write_u64(op_hash);
    for dep in dep_nums {
        hasher.write_usize(dep.0.idx);
    }
    let hash = hasher.finish();
    if let Some(i) = hash_table.get(&hash) {
        *i
    } else {
        hash_table.insert(hash, x);
        x
    }
}
// http://softlib.rice.edu/pub/CRPC-TRs/reports/CRPC-TR95636-S.pdf
pub fn global_value_numbering_pass(g: &mut GateGraphBuilder) {
    let mut numbers = NumberMap::new();

    let mut hash_table = HashMap::new();
    let mut visited = HashSet::new();
    let op_hash_offset = g.nodes.len() as u64;
    loop {
        let mut done = true;
        let mut work: VecDeque<GateIndex> = g.lever_handles.iter().copied().collect();
        work.push_back(OFF);
        work.push_back(ON);

        while let Some(x) = work.pop_front() {
            if visited.contains(&x) {
                continue;
            } else {
                visited.insert(x);
            }
            // TODO ensure dependencies are sorted at all times.
            g.get_mut(x).dependencies.sort();
            let gate = g.get(x);
            let op = gate.ty;
            let dep_nums = gate
                .dependencies
                .iter()
                .filter_map(|dep| numbers.get(&dep))
                .copied();
            let temp = lookup(op, op_hash_offset, dep_nums, x, &mut hash_table);
            let nx = numbers.get(&x);
            if nx.copied() != Some(ValueNumber(temp)) {
                done = false;
                numbers.insert(x, ValueNumber(temp));
            }
            work.extend(gate.dependents.iter())
        }

        if done {
            break;
        }
        visited.clear();
        hash_table.clear();
    }
    let mut temp_deps: Vec<GateIndex> = Vec::new();
    for (x, a) in numbers {
        if x == a.0 {
            continue;
        }
        temp_deps.clear();

        temp_deps.extend(g.get(x).dependents.iter());
        for dep in &temp_deps {
            g.get_mut(*dep).swap_dependency(x, a.0);
            g.get_mut(a.0).dependents.insert(*dep);
        }
        g.get_mut(x).dependents = Default::default()
    }

    dead_code_elimination_pass(g);
}
