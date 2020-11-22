use super::decoder;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("BUSMUX:{}", name)
}

// The output bus size will be the biggest of the input buses.
pub fn bus_multiplexer<S: Into<String>>(
    g: &mut GateGraph,
    address: &[GateIndex],
    inputs: &[&[GateIndex]],
    name: S,
) -> Vec<GateIndex> {
    let name = mkname(name.into());

    let width = inputs.iter().map(|i| i.len()).max().unwrap_or(0);
    let out: Vec<_> = (0..width).map(|_| g.or(name.clone())).collect();

    let decoded = decoder(g, address, name.clone());

    for (input, input_enabled) in inputs.iter().zip(decoded) {
        for (bit, big_or) in input.iter().zip(out.iter()) {
            let and = g.and2(*bit, input_enabled, name.clone());
            g.dpush(*big_or, and);
        }
    }
    out
}
