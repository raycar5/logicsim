use super::{decoder, register};
use crate::graph::*;

fn mkname(name: String) -> String {
    format!("RAM:{}", name)
}

pub fn ram<S: Into<String>>(
    g: &mut GateGraph,
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
        let cell = register(g, input, clock, write, read, reset, name.clone());

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
        let mut g = &mut GateGraph::new();

        let read = g.lever("read");
        let write = g.lever("write");
        let clock = g.lever("clock");
        let reset = g.lever("reset");
        let input = WordInput::new(g, 8, "input");
        let address = WordInput::new(g, 4, "input");

        let output = ram(
            g,
            read,
            write,
            clock,
            reset,
            address.bits(),
            input.bits(),
            "ram",
        );
        let out = g.output(&output, "out");

        g.init();
        g.run_until_stable(100).unwrap();

        assert_eq!(out.u8(g), 0);

        g.set_lever(read);
        assert_eq!(out.u8(g), 255);

        g.set_lever(reset);
        g.pulse_lever(clock);
        g.reset_lever(reset);
        for a in 0..(1 << address.len()) - 1 {
            address.set(&mut g, a);
            assert_eq!(out.u8(g), 0);
        }
    }

    #[test]
    fn test_ram_write_read() {
        let g = &mut GateGraph::new();

        let read = g.lever("read");
        let write = g.lever("write");
        let clock = g.lever("clock");
        let reset = g.lever("reset");
        let input = WordInput::new(g, 4, "input");
        let address = WordInput::new(g, 4, "input");

        let output = ram(
            g,
            read,
            write,
            clock,
            reset,
            address.bits(),
            input.bits(),
            "ram",
        );
        let out = g.output(&output, "out");

        g.init();
        g.run_until_stable(100).unwrap();

        g.set_lever(reset);
        g.pulse_lever(clock);
        g.reset_lever(reset);

        assert_eq!(out.u8(g), 0);

        g.set_lever(write);
        for a in 0..(1 << address.len()) - 1 {
            address.set(g, a);
            input.set(g, a ^ a);
            g.set_lever(clock);
            g.reset_lever(clock);
            assert_eq!(out.u8(g), 0);
        }
        g.reset_lever(write);
        g.set_lever(read);

        for a in 0..(1 << address.len()) - 1 {
            address.set(g, a);
            assert_eq!(out.u8(g), a ^ a);
        }
    }
}
