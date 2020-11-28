use super::{decoder, register};
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("RAM:{}", name)
}

/// Returns the output of a piece of [RAM](https://en.wikipedia.org/wiki/Random-access_memory)
/// addressed by `address`.
// rust-analyzer makes this a non issue.
#[allow(clippy::too_many_arguments)]
pub fn ram<S: Into<String>>(
    g: &mut GateGraphBuilder,
    read: GateIndex,
    write: GateIndex,
    clock: GateIndex,
    reset: GateIndex,
    address: &[GateIndex],
    input: &[GateIndex],
    name: S,
) -> Vec<GateIndex> {
    let name = mkname(name.into());
    let outputs: Vec<_> = input.iter().map(|_| g.or(name.clone())).collect();

    let decoded = decoder(g, address, name.clone());
    for cell_enable in decoded {
        let write = g.and2(cell_enable, write, name.clone());

        let read = g.and2(cell_enable, read, name.clone());
        let cell = register(g, clock, write, read, reset, input, name.clone());

        for (ob, cb) in outputs.iter().zip(cell) {
            g.dpush(*ob, cb)
        }
    }

    outputs
}
#[cfg(test)]
mod tests {
    use super::super::WordInput;
    use super::*;

    #[test]
    fn test_ram_reset() {
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;

        let read = g.lever("read");
        let write = g.lever("write");
        let clock = g.lever("clock");
        let reset = g.lever("reset");
        let input = WordInput::new(g, 8, "input");
        let address = WordInput::new(g, 4, "input");

        let output = ram(
            g,
            read.bit(),
            write.bit(),
            clock.bit(),
            reset.bit(),
            &address.bits(),
            &input.bits(),
            "ram",
        );
        let out = g.output(&output, "out");

        let g = &mut graph.init();
        g.run_until_stable(100).unwrap();

        assert_eq!(out.u8(g), 0);

        g.set_lever(read);
        assert_eq!(out.u8(g), 0);

        g.pulse_lever_stable(reset);
        for a in 0..(1 << address.len()) - 1 {
            address.set_to(g, a);
            assert_eq!(out.u8(g), 0);
        }
    }

    #[test]
    fn test_ram_write_read() {
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;

        let read = g.lever("read");
        let write = g.lever("write");
        let clock = g.lever("clock");
        let reset = g.lever("reset");
        let input = WordInput::new(g, 4, "input");
        let address = WordInput::new(g, 4, "input");

        let output = ram(
            g,
            read.bit(),
            write.bit(),
            clock.bit(),
            reset.bit(),
            &address.bits(),
            &input.bits(),
            "ram",
        );
        let out = g.output(&output, "out");

        let g = &mut graph.init();
        g.run_until_stable(100).unwrap();

        g.pulse_lever_stable(reset);

        assert_eq!(out.u8(g), 0);

        g.set_lever(write);
        for a in 0..(1 << address.len()) - 1 {
            address.set_to(g, a);
            input.set_to(g, a ^ a);
            g.set_lever(clock);
            g.reset_lever(clock);
            assert_eq!(out.u8(g), 0);
        }
        g.reset_lever(write);
        g.set_lever(read);

        for a in 0..(1 << address.len()) - 1 {
            address.set_to(g, a);
            assert_eq!(out.u8(g), a ^ a);
        }
    }
}
