use crate::bititer::BitIter;
use crate::graph::*;

pub const CONSTANT: &str = "constant";

pub fn constant<T>(c: T) -> Vec<GateIndex> {
    let width = std::mem::size_of::<T>() * 8;
    let mut out = Vec::new();
    out.reserve(width);

    for bit in BitIter::new(c) {
        if bit {
            out.push(ON);
        } else {
            out.push(OFF)
        };
    }

    out
}

pub fn zeros(n: usize) -> Vec<GateIndex> {
    (0..n).map(|_| OFF).collect()
}
pub fn ones(n: usize) -> Vec<GateIndex> {
    (0..n).map(|_| ON).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant() {
        let g = GateGraph::new();

        let constants = [0, 0b1u8, 0b10010010];
        let results = [
            [false, false, false, false, false, false, false, false],
            [true, false, false, false, false, false, false, false],
            [false, true, false, false, true, false, false, true],
        ];
        for (c, result) in constants.iter().zip(results.iter()) {
            let output: Vec<_> = constant(*c);

            for (bit, expected) in output.iter().zip(result) {
                assert_eq!(g.value(*bit), *expected)
            }
        }
    }
}