use super::{super::instruction_set::InstructionType::*, OutputType, Program};

pub struct HelloWorld();
impl Program for HelloWorld {
    fn clock_print_interval(&self) -> u64 {
        std::u64::MAX
    }
    fn output_type(&self) -> OutputType {
        OutputType::Text
    }
    fn ram_address_space_bits(&self) -> usize {
        0
    }
    fn rom(&self) -> Vec<u16> {
        // Look ma, no assembler.

        let text = "Hello World";
        let far_jmp = 16;
        let text_start = far_jmp + 2;
        let mut rom_data = vec![
            LIB.with_data(text_start).into(),
            LDR.with_0().into(),
            JZ.with_data(far_jmp).into(),
            OUT.with_0().into(),
            LIA.with_data(1).into(),
            ADD.with_0().into(),
            SWP.with_0().into(),
            JMP.with_data(2).into(),
            JMP.with_data(far_jmp).into(),
        ];
        rom_data.extend(text.chars().collect::<Vec<_>>().chunks(2).map(|c| {
            if c.len() == 2 {
                u16::from_ne_bytes([c[0] as u8, c[1] as u8])
            } else {
                c[0] as u16
            }
        }));
        rom_data
    }
}
