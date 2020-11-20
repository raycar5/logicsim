use super::adder;
use crate::graph::*;

pub const JK_FLIP_FLOP: &str = "jk_flip_flop";

pub fn jk_flip_flop(
    g: &mut GateGraph,
    reset: GateIndex,
    set: GateIndex,
    clock: GateIndex,
) -> GateIndex {
    let nclock = g.not1(clock, JK_FLIP_FLOP);

    let master_reset_and = g.nand2(reset, clock, JK_FLIP_FLOP);
    let master_set_and = g.nand2(set, clock, JK_FLIP_FLOP);

    let master_q = g.nor1(master_reset_and, JK_FLIP_FLOP);
    let master_nq = g.nor2(master_set_and, master_q, JK_FLIP_FLOP);

    g.dpush(master_q, master_nq);

    let slave_reset_and = g.and2(master_q, nclock, JK_FLIP_FLOP);
    let slave_set_and = g.and2(master_nq, nclock, JK_FLIP_FLOP);

    let slave_q = g.nor1(slave_reset_and, JK_FLIP_FLOP);
    let slave_nq = g.nor2(slave_set_and, slave_q, JK_FLIP_FLOP);

    g.dpush(slave_q, slave_nq);
    g.dpush(master_reset_and, slave_q);
    g.dpush(master_set_and, slave_nq);

    slave_q
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jk_flip_flop_set_reset() {
        let mut g = GateGraph::new();
        let set = g.lever("set");
        let reset = g.lever("reset");
        let clock = g.lever("clock");

        let q = jk_flip_flop(&mut g, reset, set, clock);

        g.init();
        g.run_until_stable(10).unwrap();

        assert_eq!(g.value(q), false);

        g.set_lever(set);
        assert_eq!(g.value(q), false);

        g.set_lever(clock);
        g.reset_lever(clock);
        g.run_until_stable(10).unwrap();
        assert_eq!(g.value(q), true);

        g.reset_lever(set);
        assert_eq!(g.value(q), true);

        g.set_lever(reset);
        assert_eq!(g.value(q), true);

        g.set_lever(clock);
        g.reset_lever(clock);
        assert_eq!(g.value(q), false);

        g.reset_lever(reset);
        assert_eq!(g.value(q), false);
    }
    #[test]
    fn test_jk_flip_flop_toggle() {
        let mut g = GateGraph::new();
        let set = g.lever("set");
        let reset = g.lever("reset");
        let clock = g.lever("clock");

        let q = jk_flip_flop(&mut g, reset, set, clock);
        g.init();
        g.run_until_stable(10).unwrap();

        assert_eq!(g.value(q), false);

        g.set_lever(set);
        g.set_lever(reset);
        assert_eq!(g.value(q), false);

        for i in 0..10 {
            g.set_lever(clock);
            g.run_until_stable(10).unwrap();
            g.reset_lever(clock);
            g.run_until_stable(10).unwrap();
            assert_eq!(g.value(q), i % 2 == 0);
        }
        g.run_until_stable(10).unwrap();
        assert_eq!(g.value(q), false);
    }
}
