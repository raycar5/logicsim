use super::adder;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("ALU:{}", name)
}

pub fn alu<S: Into<String>>(
    g: &mut GateGraph,
    cin: GateIndex,
    write_to_bus: GateIndex,
    invert_input_2: GateIndex,
    input1: &[GateIndex],
    input2: &[GateIndex],
    name: S,
) -> Vec<GateIndex> {
    let name = mkname(name.into());

    let new_input2: Vec<_> = input2
        .iter()
        .map(|i| g.xor2(*i, invert_input_2, name.clone()))
        .collect();

    adder(g, cin, input1, &new_input2, name.clone())
        .into_iter()
        .map(|out| g.and2(out, write_to_bus, name.clone()))
        .collect()
}
