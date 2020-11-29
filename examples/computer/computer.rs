use super::control_logic::*;
use logicsim::*;

pub struct ComputerIO {
    pub ig: InitializedGateGraph,
    pub clock: LeverHandle,
    pub reset: LeverHandle,
    pub ack: LeverHandle,
    pub input: WordInput,
    pub write_input: LeverHandle,
    pub input_busy: OutputHandle,
    pub output: OutputHandle,
    pub output_updated: OutputHandle,
}

pub fn mk_computer(rom_in: &[u16], ram_address_space: usize) -> ComputerIO {
    let mut graph = GateGraphBuilder::new();
    let g = &mut graph;
    let bits = 8;

    let bus = Bus::new(g, bits, "main_bus");
    wire!(g, clock);
    wire!(g, reset);
    let clock_lever = clock.make_lever(g);
    let reset_lever = reset.make_lever(g);
    let ack_lever = g.lever("ack");
    let nclock = g.not1(clock.bit(), "nclock");

    let mut rom_data = Vec::new();
    for word in rom_in {
        rom_data.extend_from_slice(&word.to_ne_bytes())
    }

    let signals = ControlSignalsSet::new(g);

    // PROGRAM COUNTER
    let pc_output = counter(
        g,
        clock.bit(),
        signals.pc_enable().bit(),
        signals.jmp().bit(),
        signals.pc_out().bit(),
        reset.bit(),
        bus.bits(),
        "pc",
    );
    bus.connect(g, &pc_output);

    // REGISTER A
    let rega_buffer = register(
        g,
        clock.bit(),
        signals.rega_in().bit(),
        ON,
        reset.bit(),
        bus.bits(),
        "rega_buffer",
    );
    let rega_output = register(g, nclock, ON, ON, reset.bit(), &rega_buffer, "rega");
    let rega_bus_output = bus_multiplexer(
        g,
        &[signals.rega_out().bit()],
        &[&zeros(bits), &rega_output],
        "rega_bus",
    );
    bus.connect(g, &rega_bus_output);

    // REGISTER B
    let regb_output = register(
        g,
        clock.bit(),
        signals.regb_in().bit(),
        ON,
        reset.bit(),
        bus.bits(),
        "regb",
    );
    let regb_bus_output = bus_multiplexer(
        g,
        &[signals.regb_out().bit()],
        &[&zeros(bits), &regb_output],
        "regb_bus",
    );
    bus.connect(g, &regb_bus_output);

    // ALU
    let alu_output = aluish(
        g,
        signals.cin().bit(),
        signals.alu_out().bit(),
        signals.alu_invert_regb().bit(),
        &rega_output,
        &regb_output,
        "alu",
    );
    bus.connect(g, &alu_output);

    // ADDRESS REGISTER
    let address_reg_output = register(
        g,
        clock.bit(),
        signals.address_reg_in().bit(),
        ON,
        reset.bit(),
        bus.bits(),
        "areg",
    );
    let address_reg_bus_output = bus_multiplexer(
        g,
        &[signals.address_reg_out().bit()],
        &[&zeros(bits), &address_reg_output],
        "areg_bus",
    );
    bus.connect(g, &address_reg_bus_output);
    // The first 2^(bits-1) addresses are ROM
    // The last 2^(bits-1) addresses are RAM
    // AKA if the last bit of the address register is set it is ROM, RAM otherwise.
    let ram_address_space_bit = address_reg_output[bits - 1];
    let rom_address_space_bit = g.not1(ram_address_space_bit, "ram_address_bit");

    // ROM
    let rom_read_enable = g.and2(
        signals.rom_out().bit(),
        rom_address_space_bit,
        "rom_read_enable",
    );
    let rom_output = rom(g, rom_read_enable, &address_reg_output, &rom_data, "rom");
    //g.probe(&rom_output, "rom");
    bus.connect(g, &rom_output);

    // RAM
    let ram_read_enable = g.and2(
        signals.ram_out().bit(),
        ram_address_space_bit,
        "ram_read_enable",
    );
    let ram_write_enable = g.and2(
        signals.ram_in().bit(),
        ram_address_space_bit,
        "ram_write_enable",
    );
    let ram_output = ram(
        g,
        ram_read_enable,
        ram_write_enable,
        clock.bit(),
        reset.bit(),
        &address_reg_output[0..ram_address_space],
        bus.bits(),
        "ram",
    );
    bus.connect(g, &ram_output);

    // OUTPUT REGISTER
    let rego_output = io_register(
        g,
        clock.bit(),
        signals.rego_in().bit(),
        ON,
        reset.bit(),
        bus.bits(),
        ack_lever.bit(),
        "rego",
    );

    // INPUT REGISTER
    let regi_input = WordInput::new(g, bits, "regi_input");
    let regi_write = g.lever("regi_write");
    let regi_clock = g.or2(clock.bit(), regi_write.bit(), "regi_clock");
    let (regi_changed, regi_output) = io_register(
        g,
        regi_clock,
        regi_write.bit(),
        signals.regi_out().bit(),
        reset.bit(),
        &regi_input.bits(),
        signals.regi_ack().bit(),
        "regi",
    );
    let regi_busy_buffer = d_flip_flop(
        g,
        regi_changed,
        nclock,
        reset.bit(),
        ON,
        ON,
        "regi_busy_buffer",
    );
    let regi_busy = g.output1(regi_busy_buffer, "regi_busy");

    bus.connect(g, &regi_output);

    let rega_zero = bus_multiplexer(g, &rega_output, &[&ones(1)], "rega_zero");
    setup_control_logic(
        g,
        rega_zero[0],
        regi_changed,
        bus.clone(),
        clock.bit(),
        reset.bit(),
        signals,
    );

    let t = std::time::Instant::now();
    let output = g.output(&rego_output.1, "output");
    let output_updated = g.output1(rego_output.0, "updated");

    let mut ig = graph.init();
    ig.run_until_stable(100).unwrap();

    // RESET
    ig.pulse_lever_stable(reset_lever);
    println!("Init+reset time: {}ms", t.elapsed().as_millis());
    println!("");

    ComputerIO {
        ig,
        clock: clock_lever,
        reset: reset_lever,
        ack: ack_lever,
        input: regi_input,
        write_input: regi_write,
        input_busy: regi_busy,
        output,
        output_updated,
    }
}
