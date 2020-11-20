use crate::graph::*;
pub const D_FLIP_FLOP: &str = "d_flip_flop";
pub fn d_flip_flop(
    g: &mut GateGraph,
    d: GateIndex,
    clock: GateIndex,
    write: GateIndex,
    read: GateIndex,
) -> GateIndex {
    let input = d;
    let clock = g.and2(clock, write, D_FLIP_FLOP);
    let ninput = g.not1(input, D_FLIP_FLOP);

    let flip_and = g.and2(ninput, clock, D_FLIP_FLOP);
    let flop_and = g.and2(input, clock, D_FLIP_FLOP);

    let q = g.nor2(flip_and, OFF, D_FLIP_FLOP);

    let nq = g.nor2(flop_and, q, D_FLIP_FLOP);
    g.d1(q, nq);
    g.and2(q, read, D_FLIP_FLOP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flip_flop() {
        let mut g = GateGraph::new();

        let d = g.lever("d");
        let read = g.lever("read");
        let write = g.lever("write");
        let clock = g.lever("clock");

        let output = d_flip_flop(&mut g, d, clock, write, read);
        g.init();

        g.run_until_stable(10).unwrap();
        assert_eq!(g.value(output), false);

        g.set_lever(d);
        g.set_lever(write);
        assert_eq!(g.value(output), false);

        g.set_lever(clock);
        g.reset_lever(clock);
        assert_eq!(g.value(output), false);

        g.set_lever(read);
        assert_eq!(g.value(output), true);

        g.reset_lever(d);
        g.reset_lever(read);
        g.reset_lever(write);
        assert_eq!(g.value(output), false);

        g.set_lever(read);
        assert_eq!(g.value(output), true);

        g.set_lever(write);
        g.set_lever(clock);

        g.reset_lever(write);
        g.reset_lever(clock);

        g.run_until_stable(10).unwrap();
        assert_eq!(g.value(output), false);
    }
}
