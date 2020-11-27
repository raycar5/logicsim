use super::decoder;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("MUX:{}", name)
}

/// Returns the a [GateIndex] representing the output of one of the `inputs` selected by `address`.
/// If `inputs` is not big enough to cover the whole address space, it will get filled by [OFF].
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
