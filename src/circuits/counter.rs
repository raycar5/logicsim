use super::{adder, bus_multiplexer, register, zeros, Bus};
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("CNTR:{}", name)
}

// rust-analyzer makes this a non issue.
#[allow(clippy::too_many_arguments)]
pub fn counter<S: Into<String>>(
    g: &mut GateGraphBuilder,
    clock: GateIndex,
    enable: GateIndex,
    write: GateIndex,
    read: GateIndex,
    reset: GateIndex,
    input: &[GateIndex],
    name: S,
) -> Vec<GateIndex> {
    let name = mkname(name.into());
    let cin = enable;

    let mut adder_input = Bus::new(g, input.len(), name.clone());
    let adder_output = adder(
        g,
        cin,
        adder_input.bits(),
        &zeros(input.len()),
        name.clone(),
    );
    let nclock = g.not1(clock, name.clone());

    let master_register_input = bus_multiplexer(g, &[write], &[&adder_output, input], name.clone());
    let master_register_output = register(
        g,
        &master_register_input,
        nclock,
        ON,
        ON,
        reset,
        name.clone(),
    );
    let slave_register_output = register(
        g,
        &master_register_output,
        clock,
        ON,
        ON,
        reset,
        name.clone(),
    );
    adder_input.connect(g, &slave_register_output);

    bus_multiplexer(
        g,
        &[read],
        &[&zeros(input.len()), &slave_register_output],
        name,
    )
}
#[cfg(test)]
mod tests {
    use super::super::constant;
    use super::*;

    #[test]
    fn test_counter_counts() {
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;

        let val = 34u8;
        let input = &constant(val)[0..2];
        let clock = g.lever("clock");
        let enable = g.lever("enable");
        let read = g.lever("read");
        let write = g.lever("write");
        let reset = g.lever("reset");

        let c = counter(
            g,
            clock.bit(),
            enable.bit(),
            write.bit(),
            read.bit(),
            reset.bit(),
            input,
            "counter",
        );
        let output = g.output(&c, "counter");

        let g = &mut graph.init();
        g.run_until_stable(100).unwrap();

        g.set_lever(reset);
        g.pulse_lever(clock);
        g.reset_lever(reset);

        assert_eq!(output.bx(g, 0), false);
        assert_eq!(output.bx(g, 1), false);

        g.set_lever(read);
        assert_eq!(output.bx(g, 0), false);
        assert_eq!(output.bx(g, 1), false);

        g.pulse_lever(clock);
        assert_eq!(output.bx(g, 0), false);
        assert_eq!(output.bx(g, 1), false);

        g.set_lever(enable);
        g.pulse_lever(clock);
        assert_eq!(output.bx(g, 0), true);
        assert_eq!(output.bx(g, 1), false);

        g.pulse_lever(clock);
        assert_eq!(output.bx(g, 0), false);
        assert_eq!(output.bx(g, 1), true);

        g.pulse_lever(clock);
        assert_eq!(output.bx(g, 0), true);
        assert_eq!(output.bx(g, 1), true);
    }
    #[test]
    fn test_counter_write() {
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;

        let val = 34u8;
        let input = &constant(val);
        let clock = g.lever("clock");
        let read = g.lever("read");
        let write = g.lever("write");
        let reset = g.lever("reset");

        let c = counter(
            g,
            clock.bit(),
            ON,
            write.bit(),
            read.bit(),
            reset.bit(),
            input,
            "counter",
        );
        let output = g.output(&c, "counter");

        let g = &mut graph.init();
        g.run_until_stable(100).unwrap();

        g.set_lever_stable(read);

        assert_eq!(output.u8(g), 255);

        g.set_lever(write);
        g.pulse_lever_stable(clock);
        g.reset_lever(write);
        g.assert_propagation(2);
        assert_eq!(output.u8(g), val);

        g.pulse_lever_stable(clock);
        assert_eq!(output.u8(g), val + 1);
    }
    #[test]
    fn test_counter_reset() {
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;

        let val = 34u8;
        let input = &constant(val);
        let clock = g.lever("clock");
        let read = g.lever("read");
        let write = g.lever("write");
        let reset = g.lever("reset");

        let c = counter(
            g,
            clock.bit(),
            ON,
            write.bit(),
            read.bit(),
            reset.bit(),
            input,
            "counter",
        );
        let output = g.output(&c, "counter");

        let g = &mut graph.init();
        g.run_until_stable(100).unwrap();

        g.set_lever(read);

        assert_eq!(output.u8(g), 0);

        for i in 0..10 {
            g.pulse_lever_stable(clock);
            assert_eq!(output.u8(g), i);
        }

        g.set_lever(reset);
        g.pulse_lever_stable(clock);

        g.assert_propagation(0);
        assert_eq!(output.u8(g), 0);
    }
}
