use crate::{graph::*, register, sr_latch};

fn mkname(name: String) -> String {
    format!("OUTREG:{}", name)
}

pub fn output_register<S: Into<String>>(
    g: &mut GateGraphBuilder,
    input: &[GateIndex],
    clock: GateIndex,
    write: GateIndex,
    read: GateIndex,
    reset: GateIndex,
    ack: GateIndex,
    name: S,
) -> (GateIndex, Vec<GateIndex>) {
    let name = mkname(name.into());

    let updated_s = g.and2(write, clock, name.clone());
    let updated_r = g.or2(reset, ack, name.clone());
    let updated_output = sr_latch(g, updated_s, updated_r, name.clone());

    let register_output = register(g, input, clock, write, read, reset, name);
    (updated_output, register_output)
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
