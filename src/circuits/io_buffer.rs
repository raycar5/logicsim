use crate::{graph::*, ram, wire, Bus, Wire, WordInput};

fn mkname(name: String) -> String {
    format!("IOBUF:{}", name)
}
/// Data structure used to represent a piece of [RAM](https://en.wikipedia.org/wiki/Random-access_memory)
/// that can be easily read and written to from Rust.
///
/// It is a naive implementation, therefore reading and writing should be done between clock cycles
/// of the circuit interacting with the IOBuffer.
// rust-analyzer makes this a non issue.
#[allow(clippy::too_many_arguments)]
pub struct IOBuffer {
    io_bus: Bus,
    address_bus: Bus,
    clock: Wire,
    read: Wire,
    write: Wire,
    reset: Wire,
    write_input: WordInput,
    read_output: OutputHandle,
    address_input: WordInput,
}
impl IOBuffer {
    /// Returns a new [IOBuffer] which stores `len` words which are `width` bits wide.
    pub fn new<S: Into<String>>(
        g: &mut GateGraphBuilder,
        width: usize,
        len: usize,
        name: S,
    ) -> Self {
        let name = mkname(name.into());
        let write_input = WordInput::new(g, width, name.clone());

        let address_bits = (len as f32).log2().floor() as usize;
        let address_input = WordInput::new(g, address_bits, name.clone());

        let io_bus = Bus::new(g, width, name.clone());
        io_bus.connect(g, &write_input.bits());

        let address_bus = Bus::new(g, address_bits, name.clone());
        address_bus.connect(g, &address_input.bits());

        let read_output = g.output(io_bus.bits(), name.clone());

        wire!(g, clock);
        wire!(g, read);
        wire!(g, write);
        wire!(g, reset);
        clock.make_lever(g);
        read.make_lever(g);
        write.make_lever(g);
        reset.make_lever(g);

        let ram_output = ram(
            g,
            read.bit(),
            write.bit(),
            clock.bit(),
            reset.bit(),
            address_bus.bits(),
            io_bus.bits(),
            name,
        );
        io_bus.connect(g, &ram_output);

        Self {
            write_input,
            address_input,
            io_bus,
            address_bus,
            read_output,
            clock,
            read,
            write,
            reset,
        }
    }
    /// Connects the IOBuffer to a circuit.
    // rust-analyzer makes this a non issue.
    #[allow(clippy::too_many_arguments)]
    pub fn connect(
        &self,
        g: &mut GateGraphBuilder,
        clock: GateIndex,
        read: GateIndex,
        write: GateIndex,
        reset: GateIndex,
        address: &[GateIndex],
        io_bus: Bus,
    ) -> Bus {
        self.address_bus.connect(g, address);
        self.clock.connect(g, clock);
        self.read.connect(g, read);
        self.write.connect(g, write);
        self.reset.connect(g, reset);
        self.io_bus.merge(g, io_bus)
    }

    /// Sets the address and write inputs to 0.
    fn reset_inputs(&self, g: &mut InitializedGateGraph) {
        self.address_input.reset(g);
        self.write_input.reset(g);
        g.run_until_stable(10).unwrap();
    }

    /// Sets word at `address` to `value`.
    /// Extra bits in `address` or `value` will be truncated.
    /// If `address` or `value` are missing bits, they will be 0 extended.
    pub fn write<A: Copy + Sized + 'static, T: Copy + Sized + 'static>(
        &self,
        g: &mut InitializedGateGraph,
        address: A,
        value: T,
    ) {
        self.write_input.set_to(g, value);
        self.address_input.set_to(g, address);

        g.set_lever(self.write.lever().unwrap());
        g.pulse_lever_stable(self.clock.lever().unwrap());
        g.reset_lever_stable(self.write.lever().unwrap());

        self.reset_inputs(g);
    }

    // TODO macro this for more types.
    /// Returns the value of the word at `address`.
    /// Extra bits in `address` will be truncated.
    /// If `address` is missing bits, it will be 0 extended.
    pub fn read_u8<A: Copy + Sized + 'static>(
        &self,
        g: &mut InitializedGateGraph,
        address: A,
    ) -> u8 {
        self.address_input.set_to(g, address);

        g.set_lever_stable(self.read.lever().unwrap());
        let output = self.read_output.u8(g);
        g.reset_lever_stable(self.read.lever().unwrap());

        self.reset_inputs(g);
        output
    }

    /// Sets all words in the buffer to 0.
    pub fn reset(&self, g: &mut InitializedGateGraph) {
        g.pulse_lever_stable(self.reset.lever().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_propagation;

    #[test]
    fn test_alone() {
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;

        let buffer = IOBuffer::new(g, 8, 2, "buffer");

        let g = &mut graph.init();
        g.run_until_stable(10).unwrap();
        buffer.reset(g);

        assert_eq!(buffer.read_u8(g, 0), 0);

        buffer.write(g, 1, 3);

        assert_eq!(buffer.read_u8(g, 0), 0);

        assert_eq!(buffer.read_u8(g, 1), 3);

        buffer.reset(g);

        assert_eq!(buffer.read_u8(g, 1), 0);
    }

    #[test]
    fn test_with_circuit() {
        let mut graph = GateGraphBuilder::new();
        let g = &mut graph;

        let width = 8;
        let len = 2;
        let address_bits = 1;

        let buffer = IOBuffer::new(g, width, len, "buffer");

        let clock = g.lever("clock");
        let read = g.lever("read");
        let write = g.lever("write");
        let reset = g.lever("reset");

        let input = WordInput::new(g, 8, "input");
        let address_input = WordInput::new(g, address_bits, "address");
        let io_bus = Bus::new(g, width, "bus");
        io_bus.connect(g, &input.bits());

        let io_bus = buffer.connect(
            g,
            clock.bit(),
            read.bit(),
            write.bit(),
            reset.bit(),
            &address_input.bits(),
            io_bus,
        );

        let output = g.output(io_bus.bits(), "output");

        let g = &mut graph.init();
        g.run_until_stable(10).unwrap();

        // Reset by circuit.
        g.pulse_lever_stable(reset);

        assert_eq!(buffer.read_u8(g, 0), 0);

        // Write in buffer, read by circuit.
        buffer.write(g, 1, 3);
        g.run_until_stable(10).unwrap();

        println!("here");
        g.set_lever_stable(read);
        assert_eq!(output.u8(g), 0);

        address_input.set_to(g, 1);
        assert_propagation!(g, 1);
        assert_eq!(output.u8(g), 3);

        // Write in circuit, read by buffer.

        g.reset_lever_stable(read);
        g.set_lever_stable(write);
        input.set_to(g, 5);
        g.pulse_lever_stable(clock);
        g.reset_lever_stable(write);

        assert_eq!(buffer.read_u8(g, 1), 5);
    }
}
