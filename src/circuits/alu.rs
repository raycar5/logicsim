use super::adder;
use crate::graph::*;

pub const ALU: &str = "alu";

pub fn alu(
    g: &mut GateGraph,
    cin: GateIndex,
    write_to_bus: GateIndex,
    invert_input_2: GateIndex,
    input1: &[GateIndex],
    input2: &[GateIndex],
) -> Vec<GateIndex> {
    let new_input2: Vec<_> = input2
        .iter()
        .map(|i| g.xor2(*i, invert_input_2, ALU))
        .collect();

    adder(g, cin, input1, &new_input2)
        .into_iter()
        .map(|out| g.and2(out, write_to_bus, ALU))
        .collect()
}
