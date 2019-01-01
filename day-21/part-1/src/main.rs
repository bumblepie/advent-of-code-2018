#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

use regex::Regex;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename).expect("Unable to open file");
    let (lines, errors): (Vec<_>, Vec<_>) = BufReader::new(file).lines().partition(Result::is_ok);
    if !errors.is_empty() {
        for error in errors.into_iter().filter_map(Result::err) {
            eprintln!("{}", error);
        }
        process::exit(1);
    }

    let lines = lines.into_iter().filter_map(Result::ok).collect();
    let (ip_register, instructions) = read_input_from_lines(lines);

    let mut state = vec![0; 6];
    let mut instruction_pointer = 0;
    let mut buf = String::new();

    println!("Ip register: {}", ip_register);
    loop {
        // Write pointer to the bound register
        state[ip_register] = instruction_pointer;
        if instruction_pointer == 8 {
            println!("\nBefore: {:?}", state);
        }

        // Fetch instruction
        if instruction_pointer >= instructions.len() {
            break;
        }
        let instruction = &instructions[instruction_pointer];
        if instruction_pointer == 8 {
            println!("Instruction {}: {:?}", instruction_pointer, instruction);
        }

        // Apply instruction
        state = apply_instruction(&state, instruction);
        if instruction_pointer == 8 {
            println!("After: {:?}", state);
        }
        // Read pointer from bound register and increment
        instruction_pointer = state[ip_register] + 1;


        if instruction_pointer == 28 {
            println!("\nFound a possible value: {:?}", state);
            std::io::stdin().read_line(&mut buf);
        }
    }
    println!("Final value of register 0: {}", state[0]);
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum OpCode {
    Addr,
    Addi,
    Mulr,
    Muli,
    Banr,
    Bani,
    Borr,
    Bori,
    Setr,
    Seti,
    Gtir,
    Gtri,
    Gtrr,
    Eqir,
    Eqri,
    Eqrr,
}

fn get_op_code_from_string(string: &str) -> Result<OpCode, String> {
    match string {
        "addr" => Ok(OpCode::Addr),
        "addi" => Ok(OpCode::Addi),
        "mulr" => Ok(OpCode::Mulr),
        "muli" => Ok(OpCode::Muli),
        "banr" => Ok(OpCode::Banr),
        "bani" => Ok(OpCode::Bani),
        "borr" => Ok(OpCode::Borr),
        "bori" => Ok(OpCode::Bori),
        "setr" => Ok(OpCode::Setr),
        "seti" => Ok(OpCode::Seti),
        "gtir" => Ok(OpCode::Gtir),
        "gtri" => Ok(OpCode::Gtri),
        "gtrr" => Ok(OpCode::Gtrr),
        "eqir" => Ok(OpCode::Eqir),
        "eqri" => Ok(OpCode::Eqri),
        "eqrr" => Ok(OpCode::Eqrr),
        _ => Err(String::from("Unknown Opcode")),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Operation {
    op_code: OpCode,
    args: (usize, usize, usize),
}

fn apply_instruction(input_state: &Vec<usize>, instruction: &Operation) -> Vec<usize> {
    let mut output_state = input_state.clone();
    let op_args = instruction.args;
    match instruction.op_code {
        OpCode::Addr => output_state[op_args.2] = input_state[op_args.0] + input_state[op_args.1],
        OpCode::Addi => output_state[op_args.2] = input_state[op_args.0] + op_args.1,
        OpCode::Mulr => output_state[op_args.2] = input_state[op_args.0] * input_state[op_args.1],
        OpCode::Muli => output_state[op_args.2] = input_state[op_args.0] * op_args.1,
        OpCode::Banr => output_state[op_args.2] = input_state[op_args.0] & input_state[op_args.1],
        OpCode::Bani => output_state[op_args.2] = input_state[op_args.0] & op_args.1,
        OpCode::Borr => output_state[op_args.2] = input_state[op_args.0] | input_state[op_args.1],
        OpCode::Bori => output_state[op_args.2] = input_state[op_args.0] | op_args.1,
        OpCode::Setr => output_state[op_args.2] = input_state[op_args.0],
        OpCode::Seti => output_state[op_args.2] = op_args.0,
        OpCode::Gtir => {
            output_state[op_args.2] = if op_args.0 > input_state[op_args.1] {
                1
            } else {
                0
            }
        },
        OpCode::Gtri => {
            output_state[op_args.2] = if input_state[op_args.0] > op_args.1 {
                1
            } else {
                0
            }
        },
        OpCode::Gtrr => {
            output_state[op_args.2] = if input_state[op_args.0] > input_state[op_args.1] {
                1
            } else {
                0
            }
        },
        OpCode::Eqir => {
            output_state[op_args.2] = if op_args.0 == input_state[op_args.1] {
                1
            } else {
                0
            }
        },
        OpCode::Eqri => {
            output_state[op_args.2] = if input_state[op_args.0] == op_args.1 {
                1
            } else {
                0
            }
        },
        OpCode::Eqrr => {
            output_state[op_args.2] = if input_state[op_args.0] == input_state[op_args.1] {
                1
            } else {
                0
            }
        },
    }
    output_state
}

fn read_input_from_lines(lines: Vec<String>) -> (usize, Vec<Operation>) {
    let mut lines = lines.into_iter();
    let header = lines.next().unwrap();

    lazy_static! {
        static ref header_regex: Regex = Regex::new(r"#ip (\d)").unwrap();
    }

    let captures = header_regex.captures(&header).unwrap();
    let ip_register = captures[1].parse::<usize>().unwrap();
    let instructions = lines
        .map(|line| {
            let sections: Vec<&str> = line.split(" ").collect();
            Operation {
                op_code: get_op_code_from_string(sections[0]).unwrap(),
                args: (
                    sections[1].parse::<usize>().unwrap(),
                    sections[2].parse::<usize>().unwrap(),
                    sections[3].parse::<usize>().unwrap(),
                ),
            }
        })
        .collect();
    (ip_register, instructions)
}
