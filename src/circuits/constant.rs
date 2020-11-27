use crate::data_structures::BitIter;
use crate::graph::*;

/// Returns a [Vec] of [ON] or [OFF] values representing the bits of any
/// [Copy] + [Sized] + ['static](https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html) `value`.
///
/// # Example
/// ```
/// # use logicsim::{GateGraphBuilder,constant};
/// # let mut g = GateGraphBuilder::new();
/// let c = constant(54u8);
///
/// let output = g.output(&c, "const");
/// let gi = &mut g.init();
///
/// assert_eq!(output.u8(gi), 54);
/// ```
pub fn constant<T: Copy + Sized + 'static>(value: T) -> Vec<GateIndex> {
    let width = std::mem::size_of::<T>() * 8;
    let mut out = Vec::new();
    out.reserve(width);

    for bit in BitIter::new(value) {
        if bit {
            out.push(ON);
        } else {
            out.push(OFF)
        };
    }

    out
}

/// Returns a [Vec] of size `n` full of [OFF].
pub fn zeros(n: usize) -> Vec<GateIndex> {
    (0..n).map(|_| OFF).collect()
}
/// Returns a [Vec] of size `n` full of [ON].
pub fn ones(n: usize) -> Vec<GateIndex> {
    (0..n).map(|_| ON).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant() {
        let constants = [0, 0b1u8, 0b10010010];
        let results = [
            [false, false, false, false, false, false, false, false],
            [true, false, false, false, false, false, false, false],
            [false, true, false, false, true, false, false, true],
        ];
        for (c, result) in constants.iter().zip(results.iter()) {
            let mut graph = GateGraphBuilder::new();
            let g = &mut graph;
            let output: Vec<_> = constant(*c);
            let out = g.output(&output, "out");

            let g = &mut graph.init();

            for (i, result) in result.iter().enumerate() {
                assert_eq!(out.bx(g, i), *result)
            }
        }
    }
}
