mod instruction_set;
mod programs;
use wires::*;
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

    let mut bus = Bus::new(g, bits, "main_bus");
    wire!(g, clock);
    wire!(g, reset);
    let clock_lever = clock.lever(g);
    let reset_lever = reset.lever(g);
    let nclock = g.not1(clock.bit(), "nclock");

    const TEXT_OUTPUT: bool = false;
    let rom_data = programs::multiply_rom(51, -2i8 as u8);
    //let rom_data = programs::echo_rom("Heya world");

    let signals = ControlSignalsSet::new(g);
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

    let rega_buffer = register(
        g,
        bus.bits(),
        clock.bit(),
        signals.rega_in().bit(),
        ON,
        reset.bit(),
        "rega_buffer",
    );
    let rega_output = register(g, &rega_buffer, nclock, ON, ON, reset.bit(), "rega");
    let rega_bus_output = bus_multiplexer(
        g,
        &[signals.rega_out().bit()],
        &[&zeros(bits), &rega_output],
        "rega_bus",
    );
    bus.connect(g, &rega_bus_output);

    let regb_output = register(
        g,
        bus.bits(),
        clock.bit(),
        signals.regb_in().bit(),
        ON,
        reset.bit(),
        "regb",
    );
    let regb_bus_output = bus_multiplexer(
        g,
        &[signals.regb_out().bit()],
        &[&zeros(bits), &regb_output],
        "regb_bus",
    );
    bus.connect(g, &regb_bus_output);

    let alu_output = alu(
        g,
        signals.cin().bit(),
        signals.alu_out().bit(),
        signals.alu_invert_regb().bit(),
        &rega_output,
        &regb_output,
        "alu",
    );
    bus.connect(g, &alu_output);

    let address_reg_output = register(
        g,
        bus.bits(),
        clock.bit(),
        signals.address_reg_in().bit(),
        ON,
        reset.bit(),
        "areg",
    );
    let address_reg_bus_output = bus_multiplexer(
        g,
        &[signals.address_reg_out().bit()],
        &[&zeros(bits), &address_reg_output],
        "areg_bus",
    );
    bus.connect(g, &address_reg_bus_output);

    let rom_output = rom(
        g,
        signals.rom_out().bit(),
        &address_reg_output,
        &rom_data,
        "rom",
    );
    bus.connect(g, &rom_output);

    let ram_output = ram(
        g,
        signals.ram_out().bit(),
        signals.ram_in().bit(),
        clock.bit(),
        reset.bit(),
        &address_reg_output[0..ram_address_space],
        bus.bits(),
        "ram",
    );
    bus.connect(g, &ram_output);

    let rego_output = register(
        g,
        bus.bits(),
        clock.bit(),
        signals.rego_in().bit(),
        ON,
        reset.bit(),
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
    let output = g.output(&rego_output, "output");
    //g.dump_dot(std::path::Path::new("computer.dot"));
    let g = &mut graph.init();
    //g.dump_dot(std::path::Path::new("computer_optimized.dot"));
    g.run_until_stable(100).unwrap();

    // RESET
    g.set_lever_stable(reset_lever);
    g.pulse_lever_stable(clock_lever);
    g.reset_lever_stable(reset_lever);
    println!("Init+reset time: {}ms", t.elapsed().as_millis());
    println!("");

    t = std::time::Instant::now();

    let mut tmavg = 10000;
    let mut old_i8 = 0;
    let mut old_char = 0 as char;
    let mut new_i8 = old_i8;
    let mut new_char = old_char;

    for i in 0..1000000 {
        g.flip_lever_stable(clock_lever);

        if TEXT_OUTPUT {
            new_char = output.char(g);
        } else {
            new_i8 = output.i8(g);
        }
        if new_i8 != old_i8 {
            old_i8 = new_i8;
            println!("output:{}, {}ns/clock", old_i8, tmavg);
        }
        if new_char != old_char {
            old_char = new_char;
            println!("output:{}, {}ns/clock", old_char, tmavg);
        }
        if i % 2 == 1 {
            tmavg = (tmavg * (i - 1) + t.elapsed().as_nanos()) / i;
            t = std::time::Instant::now();
        }
    }
    println!("{}ns/clock avg", tmavg);
}
