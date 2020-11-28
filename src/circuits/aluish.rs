use super::adder;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("ALUISH:{}", name)
}

/// Returns the output of an [ALU](https://en.wikipedia.org/wiki/Arithmetic_logic_unit) which can only add and subtract.
///
/// # Inputs
///
/// `cin` Carry in to the adder.
///
/// `read` Enables the output.
///
/// `invert_input_2` inverts the bits in `input2`.
///
/// `input1` First word input to the ALU.
///
/// `input2` Second word input to the ALU.
///
/// # Example
/// 2s complement subtraction.
/// ```
/// # use logicsim::{GateGraphBuilder,constant,aluish,ON};
/// # let mut g = GateGraphBuilder::new();
/// let input1 = constant(3i8);
/// let input2 = constant(5i8);
///
/// // Notice the carry and invert in bits are on.
/// let result = aluish(&mut g, ON, ON, ON, &input1, &input2, "alu");
/// let output = g.output(&result, "result");
///
/// let ig = &g.init();
/// assert_eq!(output.i8(ig), -2);
///
/// ```
/// # Panics
///
/// Will panic if `input1.len()` != `input2.len()`.
pub fn aluish<S: Into<String>>(
    g: &mut GateGraphBuilder,
    cin: GateIndex,
    read: GateIndex,
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
        .map(|out| g.and2(out, read, name.clone()))
        .collect()
}
