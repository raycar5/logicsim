use super::{adder, bus_multiplexer, jk_flip_flop, multiplexer, register, zeros, Bus};
use crate::graph::*;

pub const COUNTER: &str = "counter";

pub fn counter(
    g: &mut GateGraph,
    clock: GateIndex,
    enable: GateIndex,
    write: GateIndex,
    read: GateIndex,
    reset: GateIndex,
    input: &[GateIndex],
) -> Vec<GateIndex> {
    let cin = enable;

    let mut adder_input = Bus::new(g, input.len());
    let adder_output = adder(g, cin, adder_input.bits(), &zeros(input.len()));
    let nclock = g.not1(clock, COUNTER);

    let master_register_input = bus_multiplexer(g, &[write], &[&adder_output, input]);
    let master_register_output = register(g, &master_register_input, nclock, ON, ON, reset);
    let slave_register_output = register(g, &master_register_output, clock, ON, ON, reset);
    adder_input.connect(g, &slave_register_output);

    bus_multiplexer(g, &[read], &[&zeros(input.len()), &slave_register_output])
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
        g.assert_propagation(1);
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

        g.assert_propagation(0);
        assert_eq!(g.collect_u8(&output), 0);
    }
}
