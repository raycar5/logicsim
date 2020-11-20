use crate::bititer::BitIter;
use crate::graph::*;

pub const DECODER: &str = "decoder";
pub fn decoder(g: &mut GateGraph, address: &[GateIndex]) -> Vec<GateIndex> {
    let mut out = Vec::new();
    out.reserve(2usize.pow(address.len() as u32));

    let naddress: Vec<GateIndex> = address.iter().map(|bit| g.not1(*bit, DECODER)).collect();

    for i in 0..2usize.pow(address.len() as u32) {
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
        let mut g = GateGraph::new();
        let c = WordInput::new(&mut g, 8);
        let out = decoder(&mut g, &c.bits());

        g.init();
        g.run_until_stable(10).unwrap();

        assert_eq!(g.collect_u8_lossy(&out), 1);

        c.set_bit(&mut g, 0);
        assert_eq!(g.collect_u8_lossy(&out), 2);

        c.set_bit(&mut g, 1);
        assert_eq!(g.collect_u8_lossy(&out), 8);
    }
}
