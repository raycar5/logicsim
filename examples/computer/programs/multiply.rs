use super::super::instruction_set::InstructionType::*;

pub fn multiply_rom(a: u8, b: u8) -> Vec<u16> {
    // LABELS
    let number = 2;
    let start = 12;
    let end = number + 4;
    let l00p = 22;

    let ram_bit = 1 << 7;
    // RAM pointers.
    let counter = 0 | ram_bit;
    let acc = 1 | ram_bit;
    let step = 2 | ram_bit;

    vec![
        JMP.with_data(start).into(),
        a as u16,
        b as u16,
        LDA.with_data(acc).into(),
        OUT.into(),
        JMP.with_data(end + 4).into(),
        LDA.with_data(number).into(), // Program start, LOAD number 1
        STI.with_data(counter).into(),
        LDA.with_data(number + 2).into(), // LOAD number 2
        STI.with_data(acc).into(),
        STI.with_data(step).into(),
        LDA.with_data(counter).into(), // Loop start
        LIB.with_data(1).into(),
        SUB.into(),
        JZ.with_data(end).into(),
        STI.with_data(counter).into(),
        LDA.with_data(acc).into(),
        LDB.with_data(step).into(),
        ADD.into(),
        STI.with_data(acc).into(),
        JMP.with_data(l00p).into(),
    ]
}
