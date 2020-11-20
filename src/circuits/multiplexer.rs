use super::decoder;
use crate::graph::*;

pub const MULTIPLEXER: &str = "multiplexer";

pub fn multiplexer(g: &mut GateGraph, address: &[GateIndex], inputs: &[GateIndex]) -> GateIndex {
    let lines = decoder(g, address);
    let big_or = g.or(MULTIPLEXER);
    for (a, i) in lines.into_iter().zip(inputs.iter()) {
        let and = g.and2(a, *i, MULTIPLEXER);
        g.dpush(big_or, and)
    }
    big_or
}
