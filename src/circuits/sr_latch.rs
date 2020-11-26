use crate::graph::*;

fn mkname(name: String) -> String {
    format!("SRLATCH:{}", name)
}

pub fn sr_latch<S: Into<String>>(
    g: &mut GateGraphBuilder,
    s: GateIndex,
    r: GateIndex,
    name: S,
) -> GateIndex {
    let name = mkname(name.into());

    let q = g.nor2(r, OFF, name.clone());

    let nq = g.nor2(s, q, name.clone());
    g.d1(q, nq);

    q
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sr_latch() {
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;

        let s = g.lever("s");
        let r = g.lever("r");

        let output = sr_latch(g, s.bit(), r.bit(), "latchy latch");

        let out = g.output1(output, "out");
        let g = &mut graph.init();
        g.run_until_stable(10).unwrap();

        assert_eq!(out.b0(g), false);

        for i in 0..10 {
            if i % 2 == 0 {
                g.pulse_lever_stable(s);
                assert_eq!(out.b0(g), true);
            } else {
                g.pulse_lever_stable(r);
                assert_eq!(out.b0(g), false);
            }
        }
    }
}
