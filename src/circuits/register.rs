use super::{bus_multiplexer, d_flip_flop, zeros};
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("REG:{}", name)
}

pub fn register<S: Into<String>>(
    g: &mut GateGraphBuilder,
    input: &[GateIndex],
    clock: GateIndex,
    write: GateIndex,
    read: GateIndex,
    reset: GateIndex,
    name: S,
) -> Vec<GateIndex> {
    let name = mkname(name.into());

    let width = input.len();
    let mut out = Vec::new();

    let write = g.or2(write, reset, name.clone());
    let new_input = bus_multiplexer(g, &[reset], &[input, &zeros(input.len())], name.clone());
    out.reserve(width);
    for bit in new_input {
        out.push(d_flip_flop(g, bit, clock, write, read, name.clone()))
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
            &input.bits(),
            clock.bit(),
            write.bit(),
            read.bit(),
            reset.bit(),
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

        g.set_lever(reset);
        g.set_lever(clock);
        assert_eq!(out.u8(g), 0);
    }
}
