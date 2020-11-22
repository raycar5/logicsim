use super::super::instruction_set::InstructionType::*;

pub fn echo_rom(text: &str) -> Vec<u8> {
    let far_jmp = 8;
    let text_start = far_jmp + 2;
    let mut rom_data = vec![
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
    rom_data.extend(text.chars().map(|c| c as u8));
    rom_data
}
