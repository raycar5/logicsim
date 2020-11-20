use super::adder;
use crate::graph::*;

pub const ALU: &str = "alu";

pub fn alu(
    g: &mut GateGraph,
    cin: GateIndex,
    write_to_bus: GateIndex,
    input1: &[GateIndex],
    input2: &[GateIndex],
) -> Vec<GateIndex> {
    adder(g, cin, input1, input2)
        .into_iter()
        .map(|out| g.and2(out, write_to_bus, ALU))
        .collect()
}
