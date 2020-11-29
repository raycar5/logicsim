use super::{super::assembler::*, Program};
use super::{super::instruction_set::InstructionType::*, OutputType};

pub struct Greeter();
impl Program for Greeter {
    fn clock_print_interval(&self) -> u64 {
        // We don't want the clock times to interrupt our nice dialog.
        std::u64::MAX
    }
    fn output_type(&self) -> OutputType {
        OutputType::Text
    }
    fn ram_address_space_bits(&self) -> usize {
        // I mean how long is your name really?
        5
    }
    fn rom(&self) -> Vec<u16> {
        let newline = '\n' as u8;
        let hello_data = "\nWhat's your name? ".chars().map(|c| c as u8);
        let nice_to_meet_data = "Nice to meet you ".chars().map(|c| c as u8);
        assemble!(
            // Labels
            label start;
            label wait_loop;
            label out_loop;
            label exit_out;
            label out;
            label process_char;
            label process_to_out;
            label hello;
            label nice;
            label out_string;
            label end;

            // Pointers
            current_char =ram= 0;
            out_start =ram= 1;
            out_return =ram= 2;
            line_end =ram= 3;
            line_start =ram= 4;

            // Program start
            start: LIA.with_ptr(line_start);
            STI.with_ptr(line_end);

            LIA.with_label(hello);
            STI.with_ptr(out_start);

            LIA.with_label(wait_loop);
            STI.with_ptr(out_return);
            JMP.with_label(out);

            wait_loop: IN.with_label(process_char);
            JMP.with_label(wait_loop);

            // Output
            out: LDB.with_ptr(out_start);
            out_loop: LDR;
            JZ.with_label(exit_out);
            OUT;
            // We override the old data with 0s
            // if it's rom it doesn't really matter
            // since it is not writeable.
            LIA.with_0();
            STR;
            LIA.with_data(1);
            ADD;
            SWP;
            JMP.with_label(out_loop);

            exit_out: LDB.with_ptr(out_return);
            JMR;

            // If char is newline jump to out.
            process_char: STI.with_ptr(current_char);
            LIB.with_data(newline);
            SUB;
            JZ.with_label(process_to_out);
            // Otherwise store the char at *line_end
            LDB.with_ptr(line_end);
            LDA.with_ptr(current_char);
            STR;
            // And increment line_end
            LIA.with_data(1);
            ADD;
            LIB.with_ptr(line_end);
            STR;
            JMP.with_label(wait_loop);

            // Set arguments to out and jump to it.
            process_to_out:
            // First print message
            LIA.with_label(out_string);
            STI.with_ptr(out_return);
            LIA.with_label(nice);
            STI.with_ptr(out_start);
            JMP.with_label(out);

            // Then print stored string.
            out_string: LIA.with_label(end);
            STI.with_ptr(out_return);
            LIA.with_ptr(line_start);
            STI.with_ptr(out_start);
            JMP.with_label(out);

            end: JMP.with_label(start);

            data#hello: hello_data;
            NOP;
            data#nice: nice_to_meet_data;
        )
    }
}
