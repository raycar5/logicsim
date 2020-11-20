use crate::graph::*;

pub const ADDER: &str = "adder";

pub fn adder(
    g: &mut BaseNodeGraph,
    mut cin: NodeIndex,
    input1: &[NodeIndex],
    input2: &[NodeIndex],
) -> Vec<NodeIndex> {
    assert_eq!(input1.len(), input2.len());
    let bits = input1.len();
    let mut outputs = Vec::new();
    outputs.reserve(bits);
    for i in 0..bits {
        let x = g.xor2(input1[i], input2[i], ADDER);
        let output = g.xor2(x, cin, ADDER);
        let a = g.and2(input1[i], input2[i], ADDER);
        let a2 = g.and2(x, cin, ADDER);
        cin = g.or2(a2, a, ADDER);
        outputs.push(output)
    }
    outputs
}
