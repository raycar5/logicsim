use super::decoder;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("MUX:{}", name)
}

pub fn multiplexer<S: Into<String>>(
    g: &mut GateGraph,
    address: &[GateIndex],
    inputs: &[GateIndex],
    name: S,
) -> GateIndex {
    let name = mkname(name.into());

    let lines = decoder(g, address, name.clone());
    let big_or = g.or(name.clone());
    for (a, i) in lines.into_iter().zip(inputs.iter()) {
        let and = g.and2(a, *i, name.clone());
        g.dpush(big_or, and)
    }
    big_or
}
