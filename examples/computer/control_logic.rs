use super::instruction_set::InstructionType;
use std::convert::TryInto;
use strum::IntoEnumIterator;
use wires::*;

control_signal_set!(
    ControlSignalsSet,
    ram_out,
    ram_in,
    rom_out,
    pc_enable,
    alu_out,
    rega_in,
    regb_in,
    jmp,
    pc_out,
    cin,
    alu_invert_regb,
    address_reg_in,
    ir_in,
    ir_data_out,
    ic_reset
);

//
// | INSTRUCTION COUNTER | INSTRUCTION OPCODE |
// |         3 bits      |        4 bits      |
// |        b0 b1 b2     |      b3 b4 b5 b6   |
fn build_microinstructions() -> Vec<u16> {
    let mut out = vec![0; 2usize.pow(7)];
    // FIXED SECTION
    let instruction_load = [
        signals_to_bits!(ControlSignalsSet, pc_out, address_reg_in),
        signals_to_bits!(ControlSignalsSet, rom_out, ir_in, pc_enable),
    ];

    let microinstructions_per_opcode: Vec<_> = InstructionType::iter()
        .map(microinstructions_from_instruction)
        .collect();

    for instruction_step in 0..2usize.pow(3) {
        for opcode in 0..2usize.pow(4) {
            // the first 2 microinstructions are always the load
            let input = instruction_step | (opcode << 3);
            if instruction_step < 2 {
                out[input as usize] = instruction_load[instruction_step as usize];
            } else {
                let relative_i = instruction_step - 2;
                out[input] = microinstructions_per_opcode
                    .get(opcode)
                    .and_then(|ins| ins.get(relative_i))
                    .copied()
                    .unwrap_or(0);
            }
        }
    }

    out
}

fn microinstructions_from_instruction(instruction: InstructionType) -> [u16; 3] {
    use InstructionType::*;
    match instruction {
        NOP => [signals_to_bits!(ControlSignalsSet, ic_reset), 0, 0],
        LDA => [
            signals_to_bits!(ControlSignalsSet, ir_data_out, address_reg_in),
            signals_to_bits!(ControlSignalsSet, ram_out, rega_in, ic_reset),
            0,
        ],
        LOA => [
            signals_to_bits!(ControlSignalsSet, ir_data_out, address_reg_in),
            signals_to_bits!(ControlSignalsSet, rom_out, rega_in, ic_reset),
            0,
        ],
        LDB => [
            signals_to_bits!(ControlSignalsSet, ir_data_out, address_reg_in),
            signals_to_bits!(ControlSignalsSet, ram_out, regb_in, ic_reset),
            0,
        ],
        LOB => [
            signals_to_bits!(ControlSignalsSet, ir_data_out, address_reg_in),
            signals_to_bits!(ControlSignalsSet, rom_out, regb_in, ic_reset),
            0,
        ],
        LIA => [
            signals_to_bits!(ControlSignalsSet, ir_data_out, rega_in, ic_reset),
            0,
            0,
        ],
        LIB => [
            signals_to_bits!(ControlSignalsSet, ir_data_out, regb_in, ic_reset),
            0,
            0,
        ],
        ADD => [
            signals_to_bits!(ControlSignalsSet, alu_out, rega_in, ic_reset),
            0,
            0,
        ],
    }
}

pub fn setup_control_logic(
    g: &mut GateGraph,
    bits: usize,
    mut bus: Bus,
    clock: GateIndex,
    reset: GateIndex,
    mut signals: ControlSignalsSet,
) {
    let ir_output = register(g, bus.bits(), clock, signals.ir_in().bit(), ON, reset);

    let ir_data_output = bus_multiplexer(
        g,
        &[signals.ir_data_out().bit()],
        &[
            &zeros(bits / 2),
            &ir_output.iter().skip(bits / 2).copied().collect::<Vec<_>>(),
        ],
    );
    bus.connect_some(g, &ir_data_output);

    signals.ic_reset().clone().connect(g, reset);

    let nclock = g.not1(clock, "nclock");
    let instruction_counter =
        other_counter(g, nclock, ON, signals.ic_reset().bit(), ON, OFF, &zeros(3));

    let microinstruction_input: Vec<_> = instruction_counter
        .into_iter()
        .chain(ir_output.iter().take(bits / 2).copied())
        .collect();

    let microinstruction_rom_output =
        rom(g, ON, &microinstruction_input, &build_microinstructions());

    signals.connect(
        g,
        microinstruction_rom_output[0..ControlSignalsSet::len()]
            .try_into()
            .unwrap(),
    )
}
