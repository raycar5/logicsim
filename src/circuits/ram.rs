use super::{decoder, register};
use crate::graph::*;

pub const RAM: &str = "ram";

pub fn ram(
    g: &mut GateGraph,
    read: GateIndex,
    write: GateIndex,
    clock: GateIndex,
    reset: GateIndex,
    address: &[GateIndex],
    input: &[GateIndex],
) -> Vec<GateIndex> {
    let outputs: Vec<_> = input.iter().map(|_| g.or(RAM)).collect();

    let decoded = decoder(g, address);
    for cell_enable in decoded {
        let write = g.and2(cell_enable, write, RAM);

        let read = g.and2(cell_enable, read, RAM);
        let cell = register(g, input, clock, write, read, reset);

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
    use std::convert::TryInto;

    #[test]
    fn test_ram_reset() {
        let mut g = GateGraph::new();

        let read = g.lever("read");
        let write = g.lever("write");
        let clock = g.lever("clock");
        let reset = g.lever("reset");
        let input = WordInput::new(&mut g, 8);
        let address = WordInput::new(&mut g, 4);

        let output = ram(
            &mut g,
            read,
            write,
            clock,
            reset,
            address.bits(),
            input.bits(),
        )
        .try_into()
        .unwrap();

        g.init();
        g.run_until_stable(100).unwrap();

        assert_eq!(g.collect_u8(&output), 0);

        g.set_lever(read);
        assert_eq!(g.collect_u8(&output), 255);

        g.set_lever(reset);
        g.pulse_lever(clock);
        g.reset_lever(reset);
        for a in 0..2u8.pow(address.len() as u32) - 1 {
            address.set(&mut g, a);
            assert_eq!(g.collect_u8(&output), 0);
        }
    }

    #[test]
    fn test_ram_write_read() {
        let mut g = GateGraph::new();

        let read = g.lever("read");
        let write = g.lever("write");
        let clock = g.lever("clock");
        let reset = g.lever("reset");
        let input = WordInput::new(&mut g, 8);
        let address = WordInput::new(&mut g, 4);

        let output = ram(
            &mut g,
            read,
            write,
            clock,
            reset,
            address.bits(),
            input.bits(),
        )
        .try_into()
        .unwrap();

        g.init();
        g.run_until_stable(100).unwrap();

        g.set_lever(reset);
        g.pulse_lever(clock);
        g.reset_lever(reset);

        assert_eq!(g.collect_u8(&output), 0);

        g.set_lever(write);
        for a in 0..2u8.pow(address.len() as u32) - 1 {
            address.set(&mut g, a);
            input.set(&mut g, a ^ a);
            g.set_lever(clock);
            g.reset_lever(clock);
            assert_eq!(g.collect_u8(&output), 0);
        }
        g.reset_lever(write);
        g.set_lever(read);

        for a in 0..2u8.pow(address.len() as u32) - 1 {
            address.set(&mut g, a);
            assert_eq!(g.collect_u8(&output), a ^ a);
        }
    }
}
