#[macro_use]
#[allow(dead_code)]
mod assembler;
mod clock_timer;
mod computer;
mod instruction_set;
#[allow(dead_code)]
mod programs;
mod stdin_peekable;
use clock_timer::ClockTimer;
use computer::{mk_computer, ComputerIO};
use programs::{list_programs, program, OutputType};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use stdin_peekable::StdinPeekable;
#[macro_use]
extern crate strum_macros;
mod control_logic;

fn main() {
    // Handle ctrl-c
    static STOP: AtomicBool = AtomicBool::new(false);
    ctrlc::set_handler(|| STOP.store(true, Ordering::Relaxed)).unwrap();

    let program_name = std::env::args()
        .skip(1)
        .next()
        .expect("Please provide a program name as the first argument.");

    let selected_program = if let Some(p) = program(&program_name) {
        p
    } else {
        panic!(
            "Selected program not available: {}, available programs:\n{}",
            program_name,
            list_programs().join("\n")
        )
    };

    let ComputerIO {
        ack,
        clock,
        mut ig,
        write_input,
        input,
        input_busy,
        output,
        output_updated,
        ..
    } = mk_computer(
        &selected_program.rom(),
        selected_program.ram_address_space_bits(),
    );

    let ig = &mut ig;

    let mut should_reset_ack = false;
    let mut stdin = StdinPeekable::new();
    let output_type = selected_program.output_type();

    let mut timer = ClockTimer::new(selected_program.clock_print_interval());
    for i in 0..std::u32::MAX {
        if STOP.load(Ordering::Relaxed) {
            break;
        }

        ig.flip_lever_stable(clock);

        // If there's data in stdin and the computer is not busy handling input,
        // input some data.
        if let (Some(c), false) = (stdin.peek(), input_busy.b0(ig)) {
            input.set_to(ig, c);
            ig.pulse_lever_stable(write_input);
            stdin.next();
        }

        // Since the acknowledgement is synchronous we need to leave the lever on
        // during a clock cycle and then turn it off.
        if should_reset_ack {
            ig.reset_lever(ack);
            should_reset_ack = false
        }

        // If the computer has updated it's output, print it, and send an acknowledgement
        // to the computer that we have consumed its output.
        if output_updated.b0(ig) && i % 2 == 1 {
            match output_type {
                OutputType::Number => {
                    print!("{}", output.u8(ig));
                }
                OutputType::Text => {
                    print!("{}", output.char(ig));
                }
            }
            std::io::stdout().flush().unwrap();
            ig.set_lever(ack);
            should_reset_ack = true;
        }
        if i % 2 == 1 {
            // Every 2 flips it's a clock cycle.
            timer.clock();
        }
    }
}
