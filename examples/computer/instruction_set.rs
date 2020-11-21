use num_enum::TryFromPrimitive;
pub const OPCODE_LENGTH: u8 = 4;
pub const DATA_LENGTH: u8 = 4;
#[repr(u8)]
#[derive(Debug, Eq, PartialEq, EnumIter, Copy, Clone, TryFromPrimitive)]
pub enum InstructionType {
    // Do nothing
    NOP = 0,
    // Load register A from ram address.
    LDA,
    // Load register A from rom address.
    LOA,
    // Load register B from ram address.
    LDB,
    // Load register B from rom address.
    LOB,
    // Load register A with immediate value.
    LIA,
    // Load register B with immediate value.
    LIB,
    // Add the contents of register A and B and save the result in register A.
    ADD,
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

pub struct Instruction {
    ty: InstructionType,
    // Will get truncated to 4 bits.
    data: u8,
}
impl Into<u8> for Instruction {
    fn into(self) -> u8 {
        self.ty as u8 | (self.data << 4)
    }
}
