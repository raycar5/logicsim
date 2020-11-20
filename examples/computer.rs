use wires::*;
fn main() {
    let g = &mut GateGraph::new();
    let bits = 8;

    let clock = g.lever("clock");
    // TODO
    let rom_data: Vec<u8> = vec![8u8];

    let mut bus = Bus::new(g, bits);
    let reset = g.lever("reset");

    let jmp = g.lever("jmp");
    let pc_out = g.lever("pc_out");
    let pc_enable = g.lever("pc_enable");
    let pc_output = counter(g, clock, pc_enable, jmp, pc_out, reset, bus.bits());
    bus.connect(g, &pc_output);

    let rega_in = g.lever("rega_in");
    //let rega_out = g.lever("rega_out");
    let rega_output = register(g, bus.bits(), clock, rega_in, ON, reset);

    let regb_in = g.lever("regb_in");
    //let regb_out = g.lever("regb_out");
    let regb_output = register(g, bus.bits(), clock, regb_in, ON, reset);

    let cin = g.lever("cin");
    let alu_out = g.lever("alu_out");
    let alu_invert_reg_2 = g.lever("alu_invert_reg_2");
    let alu_output = alu(
        g,
        cin,
        alu_out,
        alu_invert_reg_2,
        &rega_output,
        &regb_output,
    );
    bus.connect(g, &alu_output);

    let address_reg_in = g.lever("address_reg_in");
    let address_reg_output = register(g, bus.bits(), clock, address_reg_in, ON, reset);

    let rom_out = g.lever("rom_out");
    let rom_output = rom(g, rom_out, &address_reg_output, &rom_data);
    bus.connect(g, &rom_output);

    let ram_in = g.lever("ram_in");
    let ram_out = g.lever("ram_out");
    let ram_output = ram(
        g,
        ram_out,
        ram_in,
        clock,
        reset,
        &address_reg_output,
        bus.bits(),
    );
    bus.connect(g, &ram_output);

    let instruction_reg_in = g.lever("instruction_reg_in");
    let instruction_reg_out = g.lever("instruction_reg_out");
    let instruction_reg_output = register(
        g,
        bus.bits(),
        clock,
        instruction_reg_in,
        instruction_reg_out,
        reset,
    );
    bus.connect(g, &instruction_reg_output);

    let t = std::time::Instant::now();
    g.init();
    g.run_until_stable(100).unwrap();

    g.set_lever(reset);
    g.pulse_lever(clock);
    g.reset_lever(reset);
    g.set_lever(ram_out);

    g.run_until_stable(100).unwrap();
    println!(
        "{},{}",
        g.collect_u8_lossy(&bus.bits()),
        t.elapsed().as_millis()
    );
    let t = std::time::Instant::now();
    g.reset_lever(ram_out);

    g.set_lever(rom_out);
    g.set_lever(rega_in);
    g.pulse_lever(clock);
    g.reset_lever(rom_out);
    g.reset_lever(rega_in);

    g.set_lever(alu_out);
    g.set_lever(instruction_reg_in);
    g.pulse_lever(clock);
    g.reset_lever(instruction_reg_in);

    g.set_lever(ram_in);
    g.pulse_lever(clock);
    g.reset_lever(ram_in);
    g.reset_lever(alu_out);

    g.set_lever(ram_out);

    g.run_until_stable(100).unwrap();
    println!(
        "{},{}",
        g.collect_u8_lossy(&bus.bits()),
        t.elapsed().as_millis()
    )
}
