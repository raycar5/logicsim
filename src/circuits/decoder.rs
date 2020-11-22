use crate::bititer::BitIter;
use crate::graph::*;

pub const DECODER: &str = "decoder";
pub fn decoder(g: &mut GateGraph, address: &[GateIndex]) -> Vec<GateIndex> {
    let mut out = Vec::new();
    out.reserve(1 << address.len());

    let naddress: Vec<GateIndex> = address.iter().map(|bit| g.not1(*bit, DECODER)).collect();

    for i in 0..1 << address.len() {
        let output = g.and(DECODER);
        for (bit_set, (a, na)) in BitIter::new(i).zip(address.iter().zip(naddress.iter())) {
            if bit_set {
                g.dpush(output, *a)
            } else {
                g.dpush(output, *na)
            }
        }
        out.push(output);
    }

    out
}
#[cfg(test)]
mod tests {
    use super::super::WordInput;
    use super::*;

    #[test]
    fn test_decoder() {
        let g = &mut GateGraph::new();
        let c = WordInput::new(g, 8);
        let out = decoder(g, &c.bits());
        let out = g.output(&out, "out");

        g.init();
        g.run_until_stable(10).unwrap();

        assert_eq!(out.u8(g), 1);

        c.set_bit(g, 0);
        assert_eq!(out.u8(g), 2);

        c.set_bit(g, 1);
        assert_eq!(out.u8(g), 8);
    }
}
