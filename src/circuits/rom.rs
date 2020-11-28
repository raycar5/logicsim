use super::constant;
use super::decoder::decoder;
use crate::{data_structures::BitIter, graph::*};

fn mkname(name: String) -> String {
    format!("ROM:{}", name)
}

/// Returns the output of a piece of addressable [ROM](https://en.wikipedia.org/wiki/Read-only_memory) filled with `data`.
/// If `data` is not long enough to fill the entire address space, it will be filled with [OFF].
///
/// # Example
/// ```
/// # use logicsim::{GateGraphBuilder,rom,WordInput,ON,OFF};
/// # let mut g = GateGraphBuilder::new();
/// let address = WordInput::new(&mut g, 3, "address");
/// let out = rom(&mut g, ON, &address.bits(), &[3,9,1], "rom");
///
/// let output = g.output(&out, "result");
///
/// let ig = &mut g.init();
/// ig.run_until_stable(2);
///
/// assert_eq!(output.u8(ig), 3);
///
/// address.set_to(ig, 1);
/// ig.run_until_stable(2);
/// assert_eq!(output.u8(ig), 9);
///
/// address.set_to(ig, 2);
/// ig.run_until_stable(2);
/// assert_eq!(output.u8(ig), 1);
///
/// address.set_to(ig, 3);
/// ig.run_until_stable(2);
/// assert_eq!(output.u8(ig), 0);
/// ```
///
/// # Panics
///
/// Will panic if not enough `address` bits are provided to address every value in `data`.
pub fn rom<T: Copy + 'static + Sized, S: Into<String>>(
    g: &mut GateGraphBuilder,
    read: GateIndex,
    address: &[GateIndex],
    data: &[T],
    name: S,
) -> Vec<GateIndex> {
    assert!(
        2usize.pow(address.len() as u32) >= data.len(),
        "`address` doesn't have enough bits to address every input, address bits: {} input len:{}",
        address.len(),
        data.len(),
    );
    let name = mkname(name.into());
    let word_length = std::mem::size_of::<T>() * 8;

    let decoded = decoder(g, address, name.clone());
    let out: Vec<GateIndex> = (0..word_length).map(|_| g.or(name.clone())).collect();

    for (word, d) in data.iter().zip(decoded.into_iter()) {
        // Toss a coin to your const propagator every once in a while.
        // He already has enough work.
        if BitIter::new(*word).is_zero() {
            continue;
        }
        for (or, node) in out.iter().zip(constant(*word).into_iter()) {
            let and = g.and2(d, node, name.clone());
            g.dpush(*or, and);
        }
    }

    out.into_iter()
        .map(|or| g.and2(or, read, name.clone()))
        .collect()
}
