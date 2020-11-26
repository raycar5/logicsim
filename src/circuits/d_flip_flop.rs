use crate::{graph::*, sr_latch};

fn mkname(name: String) -> String {
    format!("DFLIPFLOP:{}", name)
}

pub fn d_flip_flop<S: Into<String>>(
    g: &mut GateGraphBuilder,
    d: GateIndex,
    clock: GateIndex,
    write: GateIndex,
    read: GateIndex,
    name: S,
) -> GateIndex {
    let name = mkname(name.into());

    let input = d;
    let clock = g.and2(clock, write, name.clone());
    let ninput = g.not1(input, name.clone());

    let s_and = g.and2(input, clock, name.clone());
    let r_and = g.and2(ninput, clock, name.clone());

    let q = sr_latch(g, s_and, r_and, name.clone());
    g.and2(q, read, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flip_flop() {
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;

        let d = g.lever("d");
        let read = g.lever("read");
        let write = g.lever("write");
        let clock = g.lever("clock");

        let output = d_flip_flop(
            g,
            d.bit(),
            clock.bit(),
            write.bit(),
            read.bit(),
            "flippity floop",
        );
        let out = g.output1(output, "out");
        let g = &mut graph.init();

        g.run_until_stable(10).unwrap();
        assert_eq!(out.b0(g), false);

        g.set_lever(d);
        g.set_lever(write);
        assert_eq!(out.b0(g), false);

        g.pulse_lever_stable(clock);
        assert_eq!(out.b0(g), false);

        g.set_lever_stable(read);
        assert_eq!(out.b0(g), true);

        g.reset_lever(d);
        g.reset_lever(read);
        g.reset_lever(write);
        assert_eq!(out.b0(g), false);

        g.set_lever_stable(read);
        assert_eq!(out.b0(g), true);

        g.set_lever(write);
        g.set_lever(clock);

        g.reset_lever(write);
        g.reset_lever(clock);

        g.run_until_stable(10).unwrap();
        assert_eq!(out.b0(g), false);
    }
}
