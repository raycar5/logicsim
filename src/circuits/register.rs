use super::d_flip_flop;
use crate::graph::*;
pub const REGISTER: &str = "REGISTER";

pub fn register(
    g: &mut GateGraph,
    input: &[GateIndex],
    clock: GateIndex,
    write: GateIndex,
    read: GateIndex,
) -> Vec<GateIndex> {
    let width = input.len();
    let mut out = Vec::new();
    out.reserve(width);
    for bit in input {
        out.push(d_flip_flop(g, *bit, clock, write, read))
    }
    out
}
#[cfg(test)]
mod tests {
    use super::super::WordInput;
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_register() {
        let mut g = GateGraph::new();
        let value = 3u8;

        let input = WordInput::new(&mut g, 8);

        let read = g.lever("read");
        let write = g.lever("write");
        let clock = g.lever("clock");

        let r = register(&mut g, input.bits(), clock, write, read);

        let out = &r.clone().try_into().unwrap();
        g.init();

        input.set(&mut g, value);

        g.run_until_stable(10).unwrap();
        assert_eq!(g.collect_u8(out), 0);

        g.set_lever(write);

        g.run_until_stable(10).unwrap();
        assert_eq!(g.collect_u8(out), 0);

        g.set_lever(clock);

        g.run_until_stable(10).unwrap();
        assert_eq!(g.collect_u8(out), 0);

        g.reset_lever(clock);
        g.set_lever(read);

        g.run_until_stable(10).unwrap();
        assert_eq!(g.collect_u8(out), value);
    }
}
