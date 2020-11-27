use crate::graph::*;

fn mkname(name: String) -> String {
    format!("ADDER:{}", name)
}

/// Returns a [Vec]<[GateIndex]> representing the output of
/// a [ripple carry adder](https://en.wikipedia.org/wiki/Adder_(electronics)#Ripple-carry_adder).
///
/// Takes two inputs of any width and a carry in, the output will be the same width as the inputs.
///
/// # Example
/// ```
/// # use logicsim::{GateGraphBuilder,constant,adder,ON};
/// # let mut g = GateGraphBuilder::new();
/// let input1 = constant(3u8);
/// let input2 = constant(5u8);
///
/// // Notice the carry in bit is on.
/// let result = adder(&mut g, ON, &input1, &input2, "adder");
/// let output = g.output(&result, "result");
///
/// let ig = &g.init();
/// assert_eq!(output.u8(ig), 9);
///
/// ```
/// # Panics
///
/// Will panic if `input1.len()` != `input2.len()`.
pub fn adder<S: Into<String>>(
    g: &mut GateGraphBuilder,
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
