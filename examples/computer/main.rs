mod instruction_set;
mod programs;
use logicsim::*;
#[macro_use]
extern crate strum_macros;
#[macro_use]
mod control_logic;
use control_logic::ControlSignalsSet;

fn main() {
    let mut graph = GateGraphBuilder::new();
    let g = &mut graph;
    let bits = 8;
    let ram_address_space = 2;

    let bus = Bus::new(g, bits, "main_bus");
    wire!(g, clock);
    wire!(g, reset);
    let clock_lever = clock.make_lever(g);
    let reset_lever = reset.make_lever(g);
    let ack_lever = g.lever("ack");
    let nclock = g.not1(clock.bit(), "nclock");

    const TEXT_OUTPUT: bool = true;
    //let rom_data_u16 = programs::multiply_rom(2, 4);
    let rom_data_u16 = programs::echo_rom("Hello world");
    let mut rom_data = Vec::new();
    for word in rom_data_u16 {
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
    let ram_address_space_bit = address_reg_output[bits - 1];
    let rom_address_space_bit = g.not1(ram_address_space_bit, "ram_address_bit");

    // ROM
    let rom_read_enable = g.and2(
        signals.rom_out().bit(),
        rom_address_space_bit,
        "rom_read_enable",
    );
    let rom_output = rom(g, rom_read_enable, &address_reg_output, &rom_data, "rom");
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
    let rego_output = output_register(
        g,
        clock.bit(),
        signals.rego_in().bit(),
        ON,
        reset.bit(),
        bus.bits(),
        ack_lever.bit(),
        "rego",
    );

    let rega_zero = bus_multiplexer(g, &rega_output, &[&ones(1)], "rega_zero");
    control_logic::setup_control_logic(
        g,
        rega_zero[0],
        bus.clone(),
        clock.bit(),
        reset.bit(),
        signals,
    );

    let mut t = std::time::Instant::now();
    let output = g.output(&rego_output.1, "output");
    let output_updated = g.output1(rego_output.0, "updated");
    //g.dump_dot(std::path::Path::new("computer.dot"));
    let g = &mut graph.init();
    g.dump_dot("computer_optimized.dot");
    g.run_until_stable(100).unwrap();

    // RESET
    g.pulse_lever_stable(reset_lever);
    println!("Init+reset time: {}ms", t.elapsed().as_millis());
    println!("");

    t = std::time::Instant::now();

    let mut tmavg = 10000;
    let mut should_ack = false;

    for i in 0..10000 {
        g.flip_lever_stable(clock_lever);

        if should_ack {
            g.pulse_lever_stable(ack_lever);
            should_ack = false
        }

        if output_updated.b0(g) {
            if TEXT_OUTPUT {
                println!("output:{}, {}ns/clock", output.char(g), tmavg);
            } else {
                println!("output:{}, {}ns/clock", output.i8(g), tmavg);
            }
            should_ack = true;
        }
        if i % 2 == 1 {
            tmavg = (tmavg * (i - 1) + t.elapsed().as_nanos()) / i;
            t = std::time::Instant::now();
        }
    }
    println!("{}ns/clock avg", tmavg);
}
