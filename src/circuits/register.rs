use super::d_flip_flop;
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("REG:{}", name)
}

/// Returns the output of a [register](https://en.wikipedia.org/wiki/Hardware_register).
/// The output width will be the same as the provided `input`.
///
/// # Inputs
///
/// `clock` Clock input to the register, activated on the raising edge.
///
/// `write` If active during the `clock` raising edge, the `input` will be stored in the register.
///
/// `read` If inactive the output will be inactive.
///
/// `reset` Will set the register to zero on the raising edge. This is an async reset.
///
/// `input` Will override the contents of the register if `write` is active on the `clock` raising edge.
///
/// # Example
/// ```
/// # use logicsim::{GateGraphBuilder,register,WordInput,ON,OFF};
/// # let mut g = GateGraphBuilder::new();
/// let input = WordInput::new(&mut g, 4, "input");
/// let reset = g.lever("reset");
/// let clock = g.lever("clock");
///
/// let register_output = register(
///     &mut g,
///     clock.bit(),
///     ON,  // write
///     ON,  // read
///     reset.bit(),
///     &input.bits(),
///     "counter"
/// );
///
/// let output = g.output(&register_output, "result");
///
/// let ig = &mut g.init();
/// ig.pulse_lever_stable(reset);
///
/// assert_eq!(output.u8(ig), 0);
///
/// input.set_to(ig, 6);
/// ig.pulse_lever_stable(clock);
/// assert_eq!(output.u8(ig), 6);
///
/// input.set_to(ig, 2);
/// ig.pulse_lever_stable(clock);
/// assert_eq!(output.u8(ig), 2);
/// ```
pub fn register<S: Into<String>>(
    g: &mut GateGraphBuilder,
    clock: GateIndex,
    write: GateIndex,
    read: GateIndex,
    reset: GateIndex,
    input: &[GateIndex],
    name: S,
) -> Vec<GateIndex> {
    let name = mkname(name.into());

    let width = input.len();
    let mut out = Vec::new();

    out.reserve(width);
    for bit in input {
        out.push(d_flip_flop(
            g,
            *bit,
            clock,
            reset,
            write,
            read,
            name.clone(),
        ))
    }
    out
}
#[cfg(test)]
mod tests {
    use super::super::WordInput;
    use super::*;
    use crate::assert_propagation;

    #[test]
    fn test_register() {
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;
        let value = 3u8;

        let input = WordInput::new(g, 8, "input");

        let read = g.lever("read");
        let write = g.lever("write");
        let reset = g.lever("reset");
        let clock = g.lever("clock");

        let r = register(
            g,
            clock.bit(),
            write.bit(),
            read.bit(),
            reset.bit(),
            &input.bits(),
            "reg",
        );

        //let output =
        let out = g.output(&r, "out");

        let g = &mut graph.init();

        input.set_to(g, value);

        g.run_until_stable(10).unwrap();
        assert_eq!(out.u8(g), 0);

        g.set_lever(write);

        assert_eq!(out.u8(g), 0);

        g.set_lever_stable(clock);
        assert_eq!(out.u8(g), 0);

        g.reset_lever_stable(clock);
        g.set_lever_stable(read);
        assert_eq!(out.u8(g), value);

        g.reset_lever_stable(read);
        assert_eq!(out.u8(g), 0);

        g.set_lever_stable(read);
        assert_eq!(out.u8(g), value);

        input.set_to(g, value ^ value);
        assert_propagation!(g, 1);
        assert_eq!(out.u8(g), value);

        g.set_lever_stable(write);
        assert_eq!(out.u8(g), value);

        g.set_lever_stable(clock);
        assert_eq!(out.u8(g), value ^ value);

        g.reset_lever(clock);
        assert_eq!(out.u8(g), value ^ value);

        g.set_lever_stable(reset);
        assert_eq!(out.u8(g), 0);
    }
}
