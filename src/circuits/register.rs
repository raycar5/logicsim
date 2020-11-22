use super::{bus_multiplexer, d_flip_flop, zeros};
use crate::graph::*;
pub const REGISTER: &str = "REGISTER";

pub fn register(
    g: &mut GateGraph,
    input: &[GateIndex],
    clock: GateIndex,
    write: GateIndex,
    read: GateIndex,
    reset: GateIndex,
) -> Vec<GateIndex> {
    let width = input.len();
    let mut out = Vec::new();

    let write = g.or2(write, reset, REGISTER);
    let new_input = bus_multiplexer(g, &[reset], &[input, &zeros(input.len())]);
    out.reserve(width);
    for bit in new_input {
        out.push(d_flip_flop(g, bit, clock, write, read))
    }
    out
}
#[cfg(test)]
mod tests {
    use super::super::WordInput;
    use super::*;

    #[test]
    fn test_register() {
        let g = &mut GateGraph::new();
        let value = 3u8;

        let input = WordInput::new(g, 8);

        let read = g.lever("read");
        let write = g.lever("write");
        let reset = g.lever("reset");
        let clock = g.lever("clock");

        let r = register(g, input.bits(), clock, write, read, reset);

        //let output =
        let out = g.output(&r, "out");

        g.init();

        input.set(g, value);

        g.run_until_stable(10).unwrap();
        assert_eq!(out.u8(g), 0);

        g.set_lever(write);

        assert_eq!(out.u8(g), 0);

        g.set_lever(clock);
        assert_eq!(out.u8(g), 0);

        g.reset_lever(clock);
        g.set_lever(read);
        assert_eq!(out.u8(g), value);

        g.reset_lever(read);
        assert_eq!(out.u8(g), 0);

        g.set_lever(read);
        assert_eq!(out.u8(g), value);

        input.set(g, value ^ value);
        assert_eq!(out.u8(g), value);

        g.set_lever(write);
        assert_eq!(out.u8(g), value);

        g.set_lever(clock);
        assert_eq!(out.u8(g), value ^ value);

        g.reset_lever(clock);
        assert_eq!(out.u8(g), value ^ value);

        g.set_lever(reset);
        g.set_lever(clock);
        assert_eq!(out.u8(g), 0);
    }
}
