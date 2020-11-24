use super::constant;
use super::decoder::decoder;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("ROM:{}", name)
}

// Will fill missing addresses with zeros
pub fn rom<T: Copy, S: Into<String>>(
    g: &mut GateGraphBuilder,
    read: GateIndex,
    address: &[GateIndex],
    data: &[T],
    name: S,
) -> Vec<GateIndex> {
    let name = mkname(name.into());
    let word_length = std::mem::size_of::<T>() * 8;
    assert!(word_length <= 64);

    let decoded = decoder(g, address, name.clone());
    let out: Vec<GateIndex> = (0..word_length).map(|_| g.or(name.clone())).collect();

    for (word, d) in data.iter().zip(decoded.into_iter()) {
        for (or, node) in out.iter().zip(constant(*word).into_iter()) {
            let and = g.and2(d, node, name.clone());
            g.dpush(*or, and);
        }
    }

    out.into_iter()
        .map(|or| g.and2(or, read, name.clone()))
        .collect()
}
