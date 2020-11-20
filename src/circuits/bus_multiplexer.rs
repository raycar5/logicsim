use super::decoder;
use crate::graph::*;

pub const BUS_MULTIPLEXER: &str = "bus_multiplexer";

// The output bus size will be the biggest of the input buses.
pub fn bus_multiplexer(
    g: &mut GateGraph,
    address: &[GateIndex],
    inputs: &[&[GateIndex]],
) -> Vec<GateIndex> {
    let width = inputs.iter().map(|i| i.len()).max().unwrap_or(0);
    let out: Vec<_> = (0..width).map(|_| g.or(BUS_MULTIPLEXER)).collect();

    let decoded = decoder(g, address);

    for (input, input_enabled) in inputs.iter().zip(decoded) {
        for (bit, big_or) in input.iter().zip(out.iter()) {
            let and = g.and2(*bit, input_enabled, BUS_MULTIPLEXER);
            g.dpush(*big_or, and);
        }
    }
    out
}
