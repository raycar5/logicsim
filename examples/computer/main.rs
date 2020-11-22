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
    let far_jmp = 8;
    let text_start = far_jmp + 2;
    let mut rom_data: Vec<u8> = vec![
        LIB.with_data(text_start).into(),
        LOR.with_0().into(),
        JZ.with_data(far_jmp).into(),
        OUT.with_0().into(),
        LIA.with_data(1).into(),
        ADD.with_0().into(),
        SWP.with_0().into(),
        JMP.with_data(1).into(),
        OUT.with_0().into(),
        JMP.with_data(far_jmp).into(),
    ];
    rom_data.extend("Heya world".chars().map(|c| c as u8));

    let signals = ControlSignalsSet::new(g);
    let pc_output = counter(
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
    let rega_bus_output = bus_multiplexer(
        g,
        &[signals.rega_out().bit()],
        &[&zeros(bits), &rega_output],
    );
    bus.connect(g, &rega_bus_output);

    let regb_output = register(
        g,
        bus.bits(),
        clock.bit(),
        signals.regb_in().bit(),
        ON,
        reset.bit(),
    );
    let regb_bus_output = bus_multiplexer(
        g,
        &[signals.regb_out().bit()],
        &[&zeros(bits), &regb_output],
    );
    bus.connect(g, &regb_bus_output);

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
    let address_reg_bus_output = bus_multiplexer(
        g,
        &[signals.address_reg_out().bit()],
        &[&zeros(bits), &address_reg_output],
    );
    bus.connect(g, &address_reg_bus_output);

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
    let output = g.get(&rego_output, "output");
    g.init();
    g.run_until_stable(100).unwrap();

    // RESET
    g.set_lever_stable(reset_lever);
    g.pulse_lever_stable(clock_lever);
    g.reset_lever_stable(reset_lever);
    println!("RESET");
    println!("");

    let mut out = 'b';
    let mut tavg = 100;
    for i in 0..500 {
        g.flip_lever_stable(clock_lever);

        let new_out = output.char(g);
        if new_out != out {
            out = new_out;
            println!("output:{}, {}ms/clock", out, tavg);
        }
        tavg = (tavg + t.elapsed().as_millis()) / 2;
        t = std::time::Instant::now();
    }
}
