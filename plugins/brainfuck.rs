#![feature(collections)] // for escape_default()
extern crate "zaldinar-core" as zaldinar;

use std::borrow::IntoCow;
use std::borrow::Cow;

use zaldinar::client::PluginRegister;
use zaldinar::events::CommandEvent;

const MAX_ITERATIONS: u32 = 134217728u32;
const MAX_OUTPUT: usize = 256usize;

#[derive(Debug)]
enum Instruction {
    /// Increment the memory pointer by one
    MoveRight,
    /// Decrement the memory pointer by one
    MoveLeft,
    /// Increment the memory value at the memory pointer by one
    Increment,
    /// Decrement the memory value at the memory pointer by one
    Decrement,
    /// Output the value of the current memory pointer as a char
    Output,
    /// This is the left side of a loop.
    /// If the memory value at the memory pointer is zero, set the next instruction to the
    /// contained value.
    JumpToLeft(usize),
    /// This is the right side of a loop.
    /// If the memory value at the memory pointer is non-zero, set the next instruction to the
    /// contained value.
    JumpToRight(usize),
}

fn parse_instructions(event: &CommandEvent) -> Result<Vec<Instruction>, Cow<'static, str>> {
    // Vec of opening jumps waiting for a closing jump to find
    // each u16 is a position in the instructions vec.
    let mut waiting_opening_jumps = Vec::new();
    let mut instructions = Vec::new();
    let mut current_parsing_position = 0u64;
    for arg in &event.args {
        for c in arg.chars() {
            current_parsing_position += 1;
            let instruction = match c {
                '>' => Instruction::MoveRight,
                '<' => Instruction::MoveLeft,
                '+' => Instruction::Increment,
                '-' => Instruction::Decrement,
                '.' => Instruction::Output,
                ',' => {
                    return Err(format!("Error: Unsupported command `,` found at position {}.",
                        current_parsing_position).into_cow());
                },
                '[' => {
                    // instructions.len() is the position where JumpTo is going to end up
                    waiting_opening_jumps.push(instructions.len());
                    // This is a placeholder, this is guaranteed to be replaced when the
                    // corresponding `]` is found.
                    Instruction::JumpToLeft(0usize)
                },
                ']' => {
                    match waiting_opening_jumps.pop() {
                        Some(left_jump) => {
                            // instructions.len() is the position where the right JumpTo
                            instructions[left_jump] = Instruction::JumpToLeft(instructions.len());
                            Instruction::JumpToRight(left_jump)
                        },
                        None => {
                            return Err(format!("Error: Unbalanced `]` found at position {}, \
                                                no matching `[` found.",
                                current_parsing_position).into_cow());
                        }
                    }
                },
                _ => continue, // treat invalid characters as comments
            };
            instructions.push(instruction);
        }
        current_parsing_position += 1; // because each argument was originally separated by a space
    }

    if !waiting_opening_jumps.is_empty() {
        return Err("Error: Unbalanced `[`, no matching `]` found.".into_cow());
    }

    return Ok(instructions);
}

fn brainfuck(event: &CommandEvent) {
    let instructions = match parse_instructions(event) {
        Ok(instructions) => instructions,
        Err(e) => {
            event.client.send_message(event.channel(), e);
            return;
        }
    };

    // Program memory, max size is 2^15
    let mut memory = [0u8; 32768];
    // Current position in memory
    let mut memory_position = 0u16;
    // Next instruction to run
    let mut next_instruction = 0usize;
    // Output string buffer
    let mut output = String::new();
    // Whether or not we finished cleanly (if false, output error for maximum iterations reached)
    let mut done = false;

    // u32::MAX as a limit to the number of iterations to run for a single program.
    for _ in 0..MAX_ITERATIONS {
        if next_instruction >= instructions.len() {
            done = true;
            break;
        }
        match instructions[next_instruction] {
            Instruction::MoveRight => {
                memory_position += 1;
                memory_position %= 32768;
            },
            Instruction::MoveLeft => {
                memory_position -= 1;
                memory_position %= 32768;
            },
            Instruction::Increment => memory[memory_position as usize] += 1,
            Instruction::Decrement => memory[memory_position as usize] -= 1,
            Instruction::Output => {
                output.push(memory[memory_position as usize] as char);

                if output.len() > MAX_OUTPUT {
                    event.client.send_message(event.channel(),
                            "Reached maximum output length. (256)");
                    done = true;
                    break;
                }
            },
            Instruction::JumpToLeft(target_position) => {
                if memory[memory_position as usize] == 0 {
                    next_instruction = target_position;
                    continue; // this avoids the automatic incrementing of next_instruction below.
                }
            },
            Instruction::JumpToRight(target_position) => {
                if memory[memory_position as usize] != 0 {
                    next_instruction = target_position;
                    continue; // this avoids the automatic incrementing of next_instruction below.
                }
            },
        }
        next_instruction += 1;
    }

    if !done {
        event.client.send_message(event.channel(), "Reached maximum iterations. (134217728)");
    }

    if output.is_empty() {
        event.client.send_message(event.channel(), "No output produced.");
    } else {
        event.client.send_message(event.channel(), format!("Output: {}", output.escape_default()));
    }
}

pub fn register(register: &mut PluginRegister) {
    register.register_command("brainfuck", brainfuck);
}
