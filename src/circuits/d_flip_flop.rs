use crate::{graph::*, sr_latch};

fn mkname(name: String) -> String {
    format!("DFLIPFLOP:{}", name)
}

/// Returns the Q output of a [D flip-flop](https://en.wikipedia.org/wiki/Flip-flop_(electronics)#D_flip-flop).
///
/// # Inputs
///
/// `d` Value to store.
///
/// `clock` Stores the value `d` on the rising edge if `write` is active.
///
/// `reset` Stores the value false on the rising edge. This is an async reset.
///
/// `write` Write enable.
///
/// `read` If inactive, the output is inactive.
//
/// # Example
/// ```
/// # use logicsim::{GateGraphBuilder,d_flip_flop,ON,OFF};
/// # let mut g = GateGraphBuilder::new();
/// let d = g.lever("d");
/// let reset = g.lever("reset");
/// let clock = g.lever("clock");
/// let write = g.lever("write");
///
/// let q = d_flip_flop(
///     &mut g,
///     d.bit(),
///     clock.bit(),
///     reset.bit(),
///     write.bit(),
///     ON,  // read
///     "counter"
/// );
///
/// let output = g.output1(q, "result");
///
/// let ig = &mut g.init();
/// ig.pulse_lever_stable(reset);
///
/// assert_eq!(output.b0(ig), false);
///
/// ig.set_lever(write);
/// ig.set_lever(d);
/// ig.pulse_lever_stable(clock);
/// assert_eq!(output.b0(ig), true);
///
/// ig.reset_lever(d);
/// ig.pulse_lever_stable(clock);
/// assert_eq!(output.b0(ig), false);
/// ```
pub fn d_flip_flop<S: Into<String>>(
    g: &mut GateGraphBuilder,
    d: GateIndex,
    clock: GateIndex,
    reset: GateIndex,
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

    let r_or = g.or2(r_and, reset, name.clone());

    let q = sr_latch(g, s_and, r_or, name.clone());
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
        let reset = g.lever("reset");
        let write = g.lever("write");
        let clock = g.lever("clock");

        let output = d_flip_flop(
            g,
            d.bit(),
            clock.bit(),
            reset.bit(),
            write.bit(),
            read.bit(),
            "flippity floop",
        );
        let out = g.output1(output, "out");
        let g = &mut graph.init();

        g.run_until_stable(10).unwrap();
        g.pulse_lever_stable(reset);
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
