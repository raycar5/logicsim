use super::super::assembler::*;
use super::{super::instruction_set::InstructionType::*, OutputType, Program};

// It only multiplies constants for now, binary to decimal doesn't really fit in
// rom but I might implement it in hardware.
const NUMBER1: u8 = 6;
const NUMBER2: u8 = 7;
pub struct Multiply();
impl Program for Multiply {
    fn clock_print_interval(&self) -> u64 {
        std::u64::MAX
    }
    fn output_type(&self) -> OutputType {
        OutputType::Number
    }
    fn ram_address_space_bits(&self) -> usize {
        2
    }
    fn rom(&self) -> Vec<u16> {
        assemble!(
            // LABELS
            label end;
            label end_loop;
            label l00p;
            label number1;
            label number2;

            // RAM pointers.
            counter =ram= 0;
            acc  =ram= 1;
            step =ram= 2;

            LDA.with_label(number1);
            STI.with_ptr(counter);
            LDA.with_label(number2);
            STI.with_ptr(acc);
            STI.with_ptr(step);

            l00p: LDA.with_ptr(counter); // Loop start
            LIB.with_data(1);
            SUB;
            JZ.with_label(end);
            STI.with_ptr(counter);
            LDA.with_ptr(acc);
            LDB.with_ptr(step);
            ADD;
            STI.with_ptr(acc);
            JMP.with_label(l00p);

            end:LDA.with_ptr(acc);
            OUT;
            end_loop: JMP.with_label(end_loop);


            data#number1: [NUMBER1].iter().copied();
            data#number2: [NUMBER2].iter().copied();
        )
    }
}
