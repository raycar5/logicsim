use super::decoder;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("BUSMUX:{}", name)
}

/// Returns a [Vec]<[GateIndex]> which contains the output of the `input` selected by `address`.
/// The output width will be the width of the widest of the inputs.
/// If not enough inputs are provided, the rest of the address space will be filled with [OFF].
///
/// # Example
/// ```
/// # use logicsim::{GateGraphBuilder,constant,bus_multiplexer};
/// # let mut g = GateGraphBuilder::new();
/// let input1 = constant(3u8);
/// let input2 = constant(5u8);
///
/// let address = g.lever("address");
///
/// // Notice the carry and invert in bits are on.
/// let result = bus_multiplexer(&mut g, &[address.bit()], &[&input1, &input2], "busmux");
/// let output = g.output(&result, "result");
///
/// let ig = &mut g.init();
/// ig.run_until_stable(2).unwrap();
///
/// assert_eq!(output.u8(ig), 3);
///
/// ig.flip_lever_stable(address);
/// assert_eq!(output.u8(ig), 5);
///
/// ig.flip_lever_stable(address);
/// assert_eq!(output.u8(ig), 3);
/// ```
/// # Panics
///
/// Will panic if not enough `address` bits are provided to address every `input`.
pub fn bus_multiplexer<S: Into<String>>(
    g: &mut GateGraphBuilder,
    address: &[GateIndex],
    inputs: &[&[GateIndex]],
    name: S,
) -> Vec<GateIndex> {
    assert!(
        2usize.pow(address.len() as u32) >= inputs.len(),
        "`address` doesn't have enough bits to address every input, address bits: {} input len:{}",
        address.len(),
        inputs.len(),
    );

    let name = mkname(name.into());

    let width = inputs.iter().map(|i| i.len()).max().unwrap_or(0);
    let out: Vec<_> = (0..width).map(|_| g.or(name.clone())).collect();

    let decoded = decoder(g, address, name.clone());

    for (input, input_enabled) in inputs.iter().zip(decoded) {
        for (bit, big_or) in input.iter().zip(out.iter()) {
            let and = g.and2(*bit, input_enabled, name.clone());
            g.dpush(*big_or, and);
        }
    }
    out
}
