use super::{adder, bus_multiplexer, register, zeros, Bus};
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("CNTR:{}", name)
}

/// Returns the output of a [counter](https://en.wikipedia.org/wiki/Counter_(digital)).
/// The output width will be the same as the provided `input`.
///
/// # Inputs
///
/// `clock` Clock input to the register, activated on the raising edge.
///
/// `enable` Counter enable, if it is active during a `clock` raising edge, the counter will increment.
///
/// `write` If active during the `clock` raising edge, the `input` will be stored in the internal register.
///
/// `read` If inactive the output will be inactive.
///
/// `reset` Will set the internal register to zero on the raising edge. This is an async reset.
///
/// `input` Will override the contents of the internal register if `write` is active on the `clock` raising edge.
///
/// # Example
/// ```
/// # use logicsim::{GateGraphBuilder,counter,constant,ON,OFF};
/// # let mut g = GateGraphBuilder::new();
/// let input = constant(5u8);
/// let reset = g.lever("reset");
/// let clock = g.lever("clock");
/// let write = g.lever("write");
///
/// let counter_output = counter(
///     &mut g,
///     clock.bit(),
///     ON,  // enable
///     write.bit(),
///     ON,  // read
///     reset.bit(),
///     &input,
///     "counter"
/// );
///
/// let output = g.output(&counter_output, "result");
///
/// let ig = &mut g.init();
/// ig.pulse_lever_stable(reset);
///
/// assert_eq!(output.u8(ig), 0);
///
/// ig.pulse_lever_stable(clock);
/// assert_eq!(output.u8(ig), 1);
///
/// ig.pulse_lever_stable(clock);
/// assert_eq!(output.u8(ig), 2);
///
/// ig.set_lever(write);
/// ig.pulse_lever_stable(clock);
/// ig.reset_lever_stable(write);
/// assert_eq!(output.u8(ig), 5);
///
/// ig.pulse_lever_stable(clock);
/// assert_eq!(output.u8(ig), 6);
/// ```
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

    let adder_input = Bus::new(g, input.len(), name.clone());
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
        nclock,
        ON,
        ON,
        reset,
        &master_register_input,
        name.clone(),
    );
    let slave_register_output = register(
        g,
        clock,
        ON,
        ON,
        reset,
        &master_register_output,
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
    use crate::assert_propagation;

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

        g.pulse_lever_stable(reset);

        assert_eq!(output.bx(g, 0), false);
        assert_eq!(output.bx(g, 1), false);

        g.set_lever(read);
        assert_eq!(output.bx(g, 0), false);
        assert_eq!(output.bx(g, 1), false);

        g.pulse_lever_stable(clock);
        assert_eq!(output.bx(g, 0), false);
        assert_eq!(output.bx(g, 1), false);

        g.set_lever_stable(enable);
        g.pulse_lever_stable(clock);
        assert_eq!(output.bx(g, 0), true);
        assert_eq!(output.bx(g, 1), false);

        g.pulse_lever_stable(clock);
        assert_eq!(output.bx(g, 0), false);
        assert_eq!(output.bx(g, 1), true);

        g.pulse_lever_stable(clock);
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
        assert_propagation!(g, 2);
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

        g.set_lever_stable(read);

        assert_eq!(output.u8(g), 255);

        for i in 0..10 {
            g.pulse_lever_stable(clock);
            assert_eq!(output.u8(g), i);
        }

        g.pulse_lever_stable(reset);

        assert_propagation!(g, 0);
        assert_eq!(output.u8(g), 0);
    }
}
