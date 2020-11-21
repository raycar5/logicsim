mod instruction_set;
use wires::*;
#[macro_use]
extern crate strum_macros;
#[macro_use]
mod control_logic;
use control_logic::ControlSignalsSet;
fn main() {
    let g = &mut GateGraph::new();
    let bits = 8;

    let mut bus = Bus::new(g, bits);
    wire!(g, clock);
    wire!(g, reset);
    let clock_lever = clock.lever(g);
    let reset_lever = reset.lever(g);
    let nclock = g.not1(clock.bit(), "nclock");

    // ROM INPUT
    use instruction_set::InstructionType::*;
    let rom_data: Vec<u8> = vec![
        LIA.with_data(0).into(),
        JZ.with_data(8).into(),
        LIA.with_data(10).into(),
        OUT.with_0().into(),
        0,
        0,
        0,
        20,
        LIA.with_data(5).into(),
        OUT.with_0().into(),
    ];

    let signals = ControlSignalsSet::new(g);
    let pc_output = other_counter(
        g,
        clock.bit(),
        signals.pc_enable().bit(),
        signals.jmp().bit(),
        signals.pc_out().bit(),
        reset.bit(),
        bus.bits(),
    );
    bus.connect(g, &pc_output);

    let rega_buffer = register(
        g,
        bus.bits(),
        clock.bit(),
        signals.rega_in().bit(),
        ON,
        reset.bit(),
    );
    let rega_output = register(g, &rega_buffer, nclock, ON, ON, reset.bit());

    let regb_output = register(
        g,
        bus.bits(),
        clock.bit(),
        signals.regb_in().bit(),
        ON,
        reset.bit(),
    );

    let alu_output = alu(
        g,
        signals.cin().bit(),
        signals.alu_out().bit(),
        signals.alu_invert_regb().bit(),
        &rega_output,
        &regb_output,
    );
    bus.connect(g, &alu_output);

    let address_reg_output = register(
        g,
        bus.bits(),
        clock.bit(),
        signals.address_reg_in().bit(),
        ON,
        reset.bit(),
    );

    let rom_output = rom(g, signals.rom_out().bit(), &address_reg_output, &rom_data);
    bus.connect(g, &rom_output);

    let ram_output = ram(
        g,
        signals.ram_out().bit(),
        signals.ram_in().bit(),
        clock.bit(),
        reset.bit(),
        &address_reg_output,
        bus.bits(),
    );
    bus.connect(g, &ram_output);

    let rego_output = register(
        g,
        bus.bits(),
        clock.bit(),
        signals.rego_in().bit(),
        ON,
        reset.bit(),
    );

    let rega_zero = bus_multiplexer(g, &rega_output, &[&ones(1)]);
    control_logic::setup_control_logic(
        g,
        rega_zero[0],
        bus.clone(),
        clock.bit(),
        reset.bit(),
        signals,
    );

    let mut t = std::time::Instant::now();
    g.init();
    g.run_until_stable(100).unwrap();

    // RESET
    g.set_lever_stable(reset_lever);
    g.pulse_lever_stable(clock_lever);
    g.reset_lever_stable(reset_lever);
    println!("RESET");
    println!("");

    for i in 0..50 {
        g.flip_lever_stable(clock_lever);

        if i % 2 == 1 {
            println!(
                "output:{}, {}ms/clock",
                g.collect_u8_lossy(&rego_output) as i8,
                t.elapsed().as_millis()
            );
            t = std::time::Instant::now();
        }
    }
}
