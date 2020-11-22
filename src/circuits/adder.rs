use crate::graph::*;

fn mkname(name: String) -> String {
    format!("ADDER:{}", name)
}

pub fn adder<S: Into<String>>(
    g: &mut GateGraph,
    mut cin: GateIndex,
    input1: &[GateIndex],
    input2: &[GateIndex],
    name: S,
) -> Vec<GateIndex> {
    assert_eq!(input1.len(), input2.len());
    let name = mkname(name.into());

    let bits = input1.len();
    let mut outputs = Vec::new();
    outputs.reserve(bits);
    for i in 0..bits {
        let x = g.xor2(input1[i], input2[i], name.clone());
        let output = g.xor2(x, cin, name.clone());
        let a = g.and2(input1[i], input2[i], name.clone());
        let a2 = g.and2(x, cin, name.clone());
        cin = g.or2(a2, a, name.clone());
        outputs.push(output)
    }
    outputs
}
