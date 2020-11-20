use super::{bus_multiplexer, jk_flip_flop, multiplexer, zeros};
use crate::graph::*;

pub const COUNTER: &str = "counter";

// COUNTS ON THE FALLING EDGE
pub fn counter(
    g: &mut GateGraph,
    clock: GateIndex,
    enable: GateIndex,
    write: GateIndex,
    read: GateIndex,
    reset_all: GateIndex,
    input: &[GateIndex],
) -> Vec<GateIndex> {
    let width = input.len();
    let mut out = Vec::new();
    out.reserve(width);
    let mut carry_clock = g.and2(clock, enable, COUNTER);

    let overriding = g.or2(write, reset_all, COUNTER);
    // Set all inputs to 0 if reset is on.
    let new_input = bus_multiplexer(g, &[reset_all], &[input, &zeros(width)]);

    for i in new_input {
        let ni = g.not1(i, COUNTER);

        // If we are counting we want both set and reset on.
        // If we are overriding instead of counting we want the inputs.
        let set = multiplexer(g, &[overriding], &[ON, i]);
        let reset = multiplexer(g, &[overriding], &[ON, ni]);

        // If we are counting we want the result of the previous flip flop to be the clock.
        // If we are overriding we want all of the clocks to be synchronized.
        let clock_select = multiplexer(g, &[overriding], &[carry_clock, clock]);

        let q = jk_flip_flop(g, reset, set, clock_select);
        carry_clock = q;

        out.push(g.and2(read, q, COUNTER));
    }
    out
}
#[cfg(test)]
mod tests {
    use super::super::constant;
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_counter_counts() {
        let mut g = GateGraph::new();

        let val = 34u8;
        let input = &constant(val)[0..2];
        let clock = g.lever("clock");
        let enable = g.lever("enable");
        let read = g.lever("read");
        let write = g.lever("write");
        let reset = g.lever("reset");

        let output = counter(&mut g, clock, enable, write, read, reset, input);

        g.init();
        g.run_until_stable(100).unwrap();

        g.set_lever(reset);
        g.pulse_lever(clock);
        g.reset_lever(reset);

        assert_eq!(g.value(output[0]), false);
        assert_eq!(g.value(output[1]), false);

        g.set_lever(read);
        assert_eq!(g.value(output[0]), false);
        assert_eq!(g.value(output[1]), false);

        g.pulse_lever(clock);
        g.assert_propagation(0);
        assert_eq!(g.value(output[0]), false);
        assert_eq!(g.value(output[1]), false);

        g.set_lever(enable);
        g.pulse_lever(clock);
        g.assert_propagation(1);
        assert_eq!(g.value(output[0]), true);
        assert_eq!(g.value(output[1]), false);

        g.pulse_lever(clock);
        g.assert_propagation(2);
        assert_eq!(g.value(output[0]), false);
        assert_eq!(g.value(output[1]), true);

        g.pulse_lever(clock);
        g.assert_propagation(1);
        assert_eq!(g.value(output[0]), true);
        assert_eq!(g.value(output[1]), true);
    }
    #[test]
    fn test_counter_write() {
        let mut g = GateGraph::new();

        let val = 34u8;
        let input = &constant(val);
        let clock = g.lever("clock");
        let read = g.lever("read");
        let write = g.lever("write");
        let reset = g.lever("reset");

        let output = counter(&mut g, clock, ON, write, read, reset, input)
            .try_into()
            .unwrap();

        g.init();
        g.run_until_stable(100).unwrap();

        g.set_lever(read);

        assert_eq!(g.collect_u8(&output), 255);

        g.set_lever(write);
        g.pulse_lever(clock);
        g.reset_lever(write);
        g.run_until_stable(2).unwrap();
        assert_eq!(g.collect_u8(&output), val);

        g.pulse_lever(clock);
        g.run_until_stable(2).unwrap();
        assert_eq!(g.collect_u8(&output), val + 1);
    }
    #[test]
    fn test_counter_reset() {
        let mut g = GateGraph::new();

        let val = 34u8;
        let input = &constant(val);
        let clock = g.lever("clock");
        let read = g.lever("read");
        let write = g.lever("write");
        let reset = g.lever("reset");

        let output = counter(&mut g, clock, ON, write, read, reset, input)
            .try_into()
            .unwrap();

        g.init();
        g.run_until_stable(100).unwrap();

        g.set_lever(read);

        assert_eq!(g.collect_u8(&output), 255);

        for i in 0..10 {
            g.set_lever(clock);
            g.reset_lever(clock);
            g.assert_propagation_range(1..3);
            assert_eq!(g.collect_u8(&output), i);
        }

        g.set_lever(reset);
        g.pulse_lever(clock);

        g.assert_propagation(1);
        assert_eq!(g.collect_u8(&output), 0);
    }
}
