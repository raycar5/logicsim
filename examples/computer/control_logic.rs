use super::instruction_set::{InstructionType, DATA_LENGTH, OPCODE_LENGTH};
use std::convert::{TryFrom, TryInto};
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
    ic_reset,
    rego_in
);
// 16

//
// | INSTRUCTION COUNTER | IS REGA ZERO | INSTRUCTION OPCODE |
// |         3 bits      |     1bit     |        4 bits      |
// |        b0 b1 b2     |      b3      |      b4 b5 b6 b7   |
fn build_microinstructions() -> Vec<u16> {
    let mut out = vec![0; 2usize.pow(8)];
    // FIXED SECTION
    let instruction_load = [
        signals_to_bits!(ControlSignalsSet, pc_out, address_reg_in),
        signals_to_bits!(ControlSignalsSet, rom_out, ir_in, pc_enable),
    ];

    for instruction_step in 0..2usize.pow(3) {
        for rega_zero in 0..2 {
            let is_rega_zero = rega_zero == 1;
            for opcode in 0..2usize.pow(4) {
                // the first 2 microinstructions are always the load
                let input = instruction_step | (rega_zero << 3) | (opcode << 4);
                if instruction_step < 2 {
                    out[input as usize] = instruction_load[instruction_step as usize];
                } else {
                    let relative_i = instruction_step - 2;
                    if let (Ok(instruction), 0..=2) = ((opcode as u8).try_into(), relative_i) {
                        out[input] = microinstructions_from_instruction(
                            instruction,
                            relative_i,
                            is_rega_zero,
                        )
                    }
                }
            }
        }
    }

    out
}

fn microinstructions_from_instruction(
    instruction: InstructionType,
    instruction_step: usize,
    is_rega_zero: bool,
) -> u16 {
    use InstructionType::*;
    let micro = match instruction {
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
        OUT => [
            signals_to_bits!(ControlSignalsSet, alu_out, rego_in, ic_reset),
            0,
            0,
        ],
        JMP => [
            signals_to_bits!(ControlSignalsSet, ir_data_out, jmp, ic_reset),
            0,
            0,
        ],
        JZ => [
            if is_rega_zero {
                signals_to_bits!(ControlSignalsSet, ir_data_out, jmp, ic_reset)
            } else {
                signals_to_bits!(ControlSignalsSet, ic_reset)
            },
            0,
            0,
        ],
    };
    micro[instruction_step]
}

pub fn setup_control_logic(
    g: &mut GateGraph,
    rega_zero: GateIndex,
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
            &zeros(DATA_LENGTH as usize),
            &ir_output
                .iter()
                .skip(OPCODE_LENGTH as usize)
                .copied()
                .collect::<Vec<_>>(),
        ],
    );
    bus.connect_some(g, &ir_data_output);

    signals.ic_reset().clone().connect(g, reset);

    let nclock = g.not1(clock, "nclock");
    let instruction_counter = counter(g, nclock, ON, signals.ic_reset().bit(), ON, OFF, &zeros(3));

    let microinstruction_input: Vec<_> = instruction_counter
        .into_iter()
        .chain(std::iter::once(rega_zero))
        .chain(ir_output.iter().take(OPCODE_LENGTH as usize).copied())
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
