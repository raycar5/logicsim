use crate::data_structures::BitIter;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("DECODER:{}", name)
}

pub fn decoder<S: Into<String>>(
    g: &mut GateGraphBuilder,
    address: &[GateIndex],
    name: S,
) -> Vec<GateIndex> {
    let name = mkname(name.into());

    let mut out = Vec::new();
    out.reserve(1 << address.len());

    let naddress: Vec<GateIndex> = address
        .iter()
        .map(|bit| g.not1(*bit, name.clone()))
        .collect();

    for i in 0..1 << address.len() {
        let output = g.and(name.clone());
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
    use crate::assert_propagation;

    #[test]
    fn test_decoder() {
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;
        let c = WordInput::new(g, 2, "input");
        let out = decoder(g, &c.bits(), "decoder");
        let out = g.output(&out, "out");

        let g = &mut graph.init();
        g.run_until_stable(10).unwrap();

        assert_eq!(out.u8(g), 1);

        c.set_bit(g, 0);
        assert_propagation!(g, 1);
        assert_eq!(out.u8(g), 2);

        c.set_bit(g, 1);
        assert_propagation!(g, 1);
        assert_eq!(out.u8(g), 8);
    }
}
