use super::super::instruction_set::InstructionType::*;

pub fn multiply_rom(a: u8, b: u8) -> Vec<u8> {
    // LABELS
    let number = 1;
    let start = 6;
    let end = number + 2;
    let l00p = 13;

    // RAM pointers.
    let counter = 0;
    let acc = 1;
    let step = 2;

    vec![
        JMP.with_data(start).into(),
        a,
        b,
        LDA.with_data(acc).into(),
        OUT.into(),
        JMP.with_data(end + 1).into(),
        // Start program
        LIB.with_data(number).into(), // LOAD number 1
        LOR.into(),
        STI.with_data(counter).into(),
        LIB.with_data(number + 1).into(), // LOAD number 2
        LOR.into(),
        STI.with_data(acc).into(),
        STI.with_data(step).into(),
        LDA.with_data(counter).into(), // Loop start
        LIB.with_data(1).into(),
        SUB.into(),
        OUT.into(),
        JZ.with_data(end).into(),
        STI.with_data(counter).into(),
        LDA.with_data(acc).into(),
        LDB.with_data(step).into(),
        ADD.into(),
        STI.with_data(acc).into(),
        JMP.with_data(l00p).into(),
    ]
}
