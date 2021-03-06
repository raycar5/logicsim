use super::instruction_set::{InstructionType, DATA_LENGTH, OPCODE_LENGTH};
use logicsim::*;
use std::convert::TryInto;
use strum::EnumCount;

control_signal_set!(
    ControlSignalsSet,
    ram_out,
    ram_in,
    rom_out,
    pc_enable,
    alu_out,
    rega_in,
    regb_in,
    rega_out,
    regb_out,
    address_reg_out,
    jmp,
    pc_out,
    cin,
    alu_invert_regb,
    address_reg_in,
    ior_in,
    idr_in,
    idr_out,
    ic_reset,
    rego_in,
    regi_out,
    regi_ack
);
// 22

const INSTRUCTION_COUNTER_BITS: u32 = 3;
const IS_REGA_ZERO_BITS: u32 = 1;
const HAS_REGI_CHANGED_BITS: u32 = 1;
const MICROINSTRUCTION_INPUT_BITS: u32 =
    INSTRUCTION_COUNTER_BITS + IS_REGA_ZERO_BITS + HAS_REGI_CHANGED_BITS + OPCODE_LENGTH;

const IS_REGA_ZERO_OFFSET: u32 = INSTRUCTION_COUNTER_BITS;
const HAS_REGI_CHANGED_OFFSET: u32 = IS_REGA_ZERO_OFFSET + IS_REGA_ZERO_BITS;
const OPCODE_OFFSET: u32 = HAS_REGI_CHANGED_OFFSET + HAS_REGI_CHANGED_BITS;

// |                              Microinstruction input                                |
// | INSTRUCTION COUNTER | IS REGA ZERO | HAS REGI CHANGED |    INSTRUCTION OPCODE      |
// |         3 bits      |     1bit     |       1bit       |          8 bits            |
// |        b0 b1 b2     |      b3      |        b4        | b5 b6 b7 b8 b9 b10 b11 b12 |
fn build_microinstructions() -> Vec<u32> {
    let mut out = vec![0; 1 << MICROINSTRUCTION_INPUT_BITS];
    // FIXED SECTION
    let instruction_fetch = [
        signals_to_bits!(ControlSignalsSet, pc_out, address_reg_in),
        signals_to_bits!(ControlSignalsSet, rom_out, ior_in, pc_enable),
        signals_to_bits!(ControlSignalsSet, pc_out, address_reg_in),
        signals_to_bits!(ControlSignalsSet, rom_out, idr_in, pc_enable),
    ];

    for instruction_step in 0..1 << INSTRUCTION_COUNTER_BITS {
        for rega_zero in 0..1 << IS_REGA_ZERO_BITS {
            let is_rega_zero = rega_zero == 1;
            for regi_changed in 0..1 << HAS_REGI_CHANGED_BITS {
                let has_regi_changed = regi_changed == 1;
                for opcode in 0..InstructionType::COUNT {
                    let input = instruction_step
                        | (rega_zero << IS_REGA_ZERO_OFFSET)
                        | (regi_changed << HAS_REGI_CHANGED_OFFSET)
                        | (opcode << OPCODE_OFFSET);

                    // The first 2 microinstructions are always the instruction fetch.
                    if instruction_step < instruction_fetch.len() {
                        out[input as usize] = instruction_fetch[instruction_step as usize];
                    } else {
                        // Instruction step after fetch.
                        let relative_instruction_step = instruction_step - instruction_fetch.len();

                        if let (Ok(instruction), 0..=2) =
                            ((opcode as u8).try_into(), relative_instruction_step)
                        {
                            out[input] = microinstructions_from_instruction(
                                instruction,
                                relative_instruction_step,
                                is_rega_zero,
                                has_regi_changed,
                            )
                        }
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
    has_regi_changed: bool,
) -> u32 {
    use InstructionType::*;
    let micro = match instruction {
        NOP => [signals_to_bits!(ControlSignalsSet, ic_reset), 0, 0],
        LDA => [
            signals_to_bits!(ControlSignalsSet, idr_out, address_reg_in),
            signals_to_bits!(ControlSignalsSet, ram_out, rom_out, rega_in, ic_reset),
            0,
        ],
        LDB => [
            signals_to_bits!(ControlSignalsSet, idr_out, address_reg_in),
            signals_to_bits!(ControlSignalsSet, ram_out, rom_out, regb_in, ic_reset),
            0,
        ],
        LIA => [
            signals_to_bits!(ControlSignalsSet, idr_out, rega_in, ic_reset),
            0,
            0,
        ],
        LIB => [
            signals_to_bits!(ControlSignalsSet, idr_out, regb_in, ic_reset),
            0,
            0,
        ],
        LDR => [
            signals_to_bits!(ControlSignalsSet, regb_out, address_reg_in),
            signals_to_bits!(ControlSignalsSet, ram_out, rom_out, rega_in, ic_reset),
            0,
        ],
        STR => [
            signals_to_bits!(ControlSignalsSet, regb_out, address_reg_in),
            signals_to_bits!(ControlSignalsSet, rega_out, ram_in, ic_reset),
            0,
        ],
        STI => [
            signals_to_bits!(ControlSignalsSet, idr_out, address_reg_in),
            signals_to_bits!(ControlSignalsSet, rega_out, ram_in, ic_reset),
            0,
        ],
        SWP => [
            // Cheeky use of the address register which will be reset by the load of the next instruction.
            signals_to_bits!(ControlSignalsSet, rega_out, address_reg_in),
            signals_to_bits!(ControlSignalsSet, regb_out, rega_in),
            signals_to_bits!(ControlSignalsSet, address_reg_out, regb_in),
        ],
        ADD => [
            signals_to_bits!(ControlSignalsSet, alu_out, rega_in, ic_reset),
            0,
            0,
        ],
        SUB => [
            signals_to_bits!(
                ControlSignalsSet,
                alu_invert_regb,
                cin,
                alu_out,
                rega_in,
                ic_reset
            ),
            0,
            0,
        ],
        OUT => [
            signals_to_bits!(ControlSignalsSet, rega_out, rego_in, ic_reset),
            0,
            0,
        ],
        IN => {
            if has_regi_changed {
                [
                    signals_to_bits!(ControlSignalsSet, regi_out, rega_in),
                    signals_to_bits!(ControlSignalsSet, idr_out, jmp),
                    signals_to_bits!(ControlSignalsSet, regi_ack, ic_reset),
                ]
            } else {
                [signals_to_bits!(ControlSignalsSet, ic_reset), 0, 0]
            }
        }
        JMP => [
            signals_to_bits!(ControlSignalsSet, idr_out, jmp, ic_reset),
            0,
            0,
        ],
        JMR => [
            signals_to_bits!(ControlSignalsSet, regb_out, jmp, ic_reset),
            0,
            0,
        ],
        JZ => [
            if is_rega_zero {
                signals_to_bits!(ControlSignalsSet, idr_out, jmp, ic_reset)
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
    g: &mut GateGraphBuilder,
    rega_zero: GateIndex,
    regi_changed: GateIndex,
    bus: Bus,
    clock: GateIndex,
    reset: GateIndex,
    mut signals: ControlSignalsSet,
) {
    // INSTRUCTION OPCODE REGISTER
    let ior_output = register(
        g,
        clock,
        signals.ior_in().bit(),
        ON,
        reset,
        bus.bits(),
        "ior",
    );
    assert_eq!(ior_output.len(), OPCODE_LENGTH as usize);

    // INSTRUCTION DATA REGISTER
    let idr_output = register(
        g,
        clock,
        signals.idr_in().bit(),
        signals.idr_out().bit(),
        reset,
        bus.bits(),
        "idr",
    );
    assert_eq!(idr_output.len(), DATA_LENGTH as usize);

    bus.connect(g, &idr_output);

    // INSTRUCTION COUNTER
    signals.ic_reset().clone().connect(g, reset);

    let nclock = g.not1(clock, "nclock");
    let instruction_counter = counter(
        g,
        nclock,
        ON,
        signals.ic_reset().bit(),
        ON,
        reset,
        &zeros(3),
        "ic",
    );

    // MICROINSTRUCTION ROM
    let microinstruction_input: Vec<_> = instruction_counter
        .into_iter()
        .chain(std::iter::once(rega_zero))
        .chain(std::iter::once(regi_changed))
        .chain(ior_output)
        .collect();

    let microinstruction_rom_output = rom(
        g,
        ON,
        &microinstruction_input,
        &build_microinstructions(),
        "micro_rom",
    );

    signals.connect(
        g,
        microinstruction_rom_output[0..ControlSignalsSet::len()]
            .try_into()
            .unwrap(),
    )
}
