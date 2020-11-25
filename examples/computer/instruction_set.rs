use num_enum::TryFromPrimitive;
pub const OPCODE_LENGTH: u32 = 8;
pub const DATA_LENGTH: u32 = 8;
#[repr(u8)]
#[derive(Debug, Eq, PartialEq, EnumIter, Copy, Clone, TryFromPrimitive)]
pub enum InstructionType {
    // Do nothing
    //NOP = 0,
    // Load register A from ram address.
    LDA,
    // Load register B from ram address.
    LDB,
    // Load register A with immediate value.
    LIA,
    // Load register B with immediate value.
    LIB,
    // Load register A with the contents of ram pointed to by register B.
    LDR,
    // Store register A at the address in register B.
    STR,
    // Store register A at the immediate address.
    STI,
    // Swap the contents of register A and B.
    SWP,
    // Add the contents of register A and B and save the result in register A.
    ADD,
    // Substract the contents of register B from A and save the result in register A.
    SUB,
    // Load the result of the alu to the output register.
    OUT,
    // Set the program counter to address.
    JMP,
    // Set the program counter to address if register A is zero.
    JZ,
}
impl InstructionType {
    pub fn with_data(&self, data: u8) -> Instruction {
        Instruction { ty: *self, data }
    }
    pub fn with_0(&self) -> Instruction {
        Instruction { ty: *self, data: 0 }
    }
}
impl Into<u16> for InstructionType {
    fn into(self) -> u16 {
        self.with_0().into()
    }
}

pub struct Instruction {
    ty: InstructionType,
    data: u8,
}
impl Into<u16> for Instruction {
    fn into(self) -> u16 {
        self.ty as u16 | ((self.data as u16) << OPCODE_LENGTH)
    }
}
