use super::decoder;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("MUX:{}", name)
}

/// Returns the output of a [multiplexer](https://en.wikipedia.org/wiki/Multiplexer).
/// which selects one of the `inputs` by `address`.
/// If `inputs` is not big enough to cover the whole address space, it will get filled by [OFF].
///
/// # Example
/// ```
/// # use logicsim::{GateGraphBuilder,multiplexer,WordInput,ON,OFF};
/// # let mut g = GateGraphBuilder::new();
/// let address = WordInput::new(&mut g, 3, "address");
///
/// // Notice the carry and invert in bits are on.
/// let result = multiplexer(&mut g, &address.bits(), &[ON, OFF, OFF, ON], "mux");
/// let output = g.output1(result, "result");
///
/// let ig = &mut g.init();
/// ig.run_until_stable(2);
///
/// assert_eq!(output.b0(ig), true);
///
/// address.set_to(ig, 1);
/// ig.run_until_stable(2);
/// assert_eq!(output.b0(ig), false);
///
/// address.set_to(ig, 2);
/// ig.run_until_stable(2);
/// assert_eq!(output.b0(ig), false);
///
/// address.set_to(ig, 3);
/// ig.run_until_stable(2);
/// assert_eq!(output.b0(ig), true);
/// ```
///
/// # Panics
///
/// Will panic if not enough `address` bits are provided to address every `input`.
pub fn multiplexer<S: Into<String>>(
    g: &mut GateGraphBuilder,
    address: &[GateIndex],
    inputs: &[GateIndex],
    name: S,
) -> GateIndex {
    assert!(
        2usize.pow(address.len() as u32) >= inputs.len(),
        "`address` doesn't have enough bits to address every input, address bits: {} input len:{}",
        address.len(),
        inputs.len(),
    );
    let name = mkname(name.into());

    let lines = decoder(g, address, name.clone());
    let big_or = g.or(name.clone());
    for (a, i) in lines.into_iter().zip(inputs.iter()) {
        let and = g.and2(a, *i, name.clone());
        g.dpush(big_or, and)
    }
    big_or
}
