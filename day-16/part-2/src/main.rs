#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::collections::{HashMap, HashSet};
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
        for err in errors.into_iter().filter_map(Result::err) {
            eprintln!("{}", err);
        }
        process::exit(1);
    }

    let lines = lines.into_iter().filter_map(Result::ok);
    let (samples, program) = read_samples_and_program_from_lines(lines);
    let sample_ops: Vec<(Sample, HashSet<Op>)> = samples.into_iter().map(|sample| {
        let sample_map: HashMap<Op, Vec<usize>> = op_values().into_iter().map(|op| (op.clone(), output(&op, &sample.before_state, &sample.opcode[1..4].to_vec()))).collect();
        let possible_ops = sample_map.into_iter()
            .filter(|(_op, output)| *output == sample.after_state)
            .map(|(op, _output)| op)
            .collect();
        (sample, possible_ops)
    }).collect();

    let opcodes_map = sample_ops.into_iter().fold(HashMap::new(), |mut opcodes, (sample, possible_ops)| {
        let sample_opcode = sample.opcode[0];
        let all_ops = op_values().into_iter().collect();
        let current_possible_ops = opcodes.get(&sample_opcode).unwrap_or(&all_ops);
        let new_possible_ops = possible_ops.intersection(current_possible_ops)
            .map(|op| op.clone())
            .collect();
        opcodes.insert(sample_opcode, new_possible_ops);
        opcodes
    });

    let opcode_map = reduce_opcode_map(opcodes_map).expect("Failed to reduce opcode map - too many possibilities");
    let mut state = vec![0,0,0,0];
    for opcode in program {
        state = output(opcode_map.get(&opcode[0]).unwrap(), &state, &opcode[1..4].to_vec());
    }
    println!("{:?}", state);
}

#[derive(Debug)]
struct Sample {
    before_state: Vec<usize>,
    opcode: Vec<usize>,
    after_state: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum Op {
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

fn op_values() -> Vec<Op> {
    vec![Op::Addr,
         Op::Addi,
         Op::Mulr,
         Op::Muli,
         Op::Banr,
         Op::Bani,
         Op::Borr,
         Op::Bori,
         Op::Setr,
         Op::Seti,
         Op::Gtir,
         Op::Gtri,
         Op::Gtrr,
         Op::Eqir,
         Op::Eqri,
         Op::Eqrr]
}

fn output(op: &Op, input_state: &Vec<usize>, op_args: &Vec<usize>) -> Vec<usize> {
    let mut output_state = input_state.clone();
    match op {
        Op::Addr => output_state[op_args[2]] = input_state[op_args[0]] + input_state[op_args[1]],
        Op::Addi => output_state[op_args[2]] = input_state[op_args[0]] + op_args[1],
        Op::Mulr => output_state[op_args[2]] = input_state[op_args[0]] * input_state[op_args[1]],
        Op::Muli => output_state[op_args[2]] = input_state[op_args[0]] * op_args[1],
        Op::Banr => output_state[op_args[2]] = input_state[op_args[0]] & input_state[op_args[1]],
        Op::Bani => output_state[op_args[2]] = input_state[op_args[0]] & op_args[1],
        Op::Borr => output_state[op_args[2]] = input_state[op_args[0]] | input_state[op_args[1]],
        Op::Bori => output_state[op_args[2]] = input_state[op_args[0]] | op_args[1],
        Op::Setr => output_state[op_args[2]] = input_state[op_args[0]],
        Op::Seti => output_state[op_args[2]] = op_args[0],
        Op::Gtir => output_state[op_args[2]] = if op_args[0] > input_state[op_args[1]] { 1 } else { 0 },
        Op::Gtri => output_state[op_args[2]] = if input_state[op_args[0]] > op_args[1] { 1 } else { 0 },
        Op::Gtrr => output_state[op_args[2]] = if input_state[op_args[0]] > input_state[op_args[1]] { 1 } else { 0 },
        Op::Eqir => output_state[op_args[2]] = if op_args[0] == input_state[op_args[1]] { 1 } else { 0 },
        Op::Eqri => output_state[op_args[2]] = if input_state[op_args[0]] == op_args[1] { 1 } else { 0 },
        Op::Eqrr => output_state[op_args[2]] = if input_state[op_args[0]] == input_state[op_args[1]] { 1 } else { 0 },
    }
    output_state
}

fn read_samples_and_program_from_lines<I>(mut lines: I) -> (Vec<Sample>, Vec<Vec<usize>>)
where
    I: Iterator<Item = String>,
{
    lazy_static! {
        static ref before_regex: Regex =
            Regex::new(r"Before: \[(\d+), (\d+), (\d+), (\d+)\]").unwrap();
        static ref after_regex: Regex = Regex::new(r"After:  \[(\d+), (\d+), (\d+), (\d+)\]").unwrap();
    }
    let mut samples = Vec::new();
    // Read samples
    loop {
        let before_line = lines.next();
        if before_line.is_none() {
            break;
        }
        let before_line = before_line.unwrap();
        if before_line.is_empty() {
            break;
        }
        let opcode_line = lines.next().unwrap();
        let after_line = lines.next().unwrap();
        let _blank_line = lines.next();

        let before_captures = before_regex.captures(&before_line).unwrap();
        let before_state = vec![
            before_captures[1].parse::<usize>().unwrap(),
            before_captures[2].parse::<usize>().unwrap(),
            before_captures[3].parse::<usize>().unwrap(),
            before_captures[4].parse::<usize>().unwrap(),
        ];
        let after_captures = after_regex.captures(&after_line).unwrap();
        let after_state = vec![
            after_captures[1].parse::<usize>().unwrap(),
            after_captures[2].parse::<usize>().unwrap(),
            after_captures[3].parse::<usize>().unwrap(),
            after_captures[4].parse::<usize>().unwrap(),
        ];
        let opcode: Vec<usize> = opcode_line
            .split(" ")
            .map(|code| code.parse::<usize>().unwrap())
            .collect();
        samples.push(Sample {
            before_state,
            opcode,
            after_state,
        })
    }

    // Read remaining program opcode lines
    let opcodes: Vec<Vec<usize>> = lines
        .filter(|line| !line.is_empty())
        .map(|opcode_line| {
            opcode_line
                .split(" ")
                .map(|code| code.parse::<usize>().unwrap())
                .collect()
        })
        .collect();
    (samples, opcodes)
}

fn reduce_opcode_map(mut map: HashMap<usize, HashSet<Op>>) -> Result<HashMap<usize, Op>, String> {
    let mut confirmed_opcodes = HashMap::new();
    while !map.is_empty() {
        let single_values: Vec<(usize, Op)> = map.iter()
            .filter(|(_instruction, ops)| ops.len() == 1)
            .map(|(instruction, ops)| (instruction.clone(), ops.iter().next().unwrap().clone()))
            .collect();
        if single_values.is_empty() {
            return Err(String::from("Could not reduce opcode map: too many possibilities"));
        }
        for (instruction, op) in single_values {
            confirmed_opcodes.insert(instruction, op.clone());
            map.remove(&instruction);
            map = map.into_iter()
                .map(|(instruction, mut ops)| {
                    ops.remove(&op);
                    (instruction, ops)
                }).collect();
        }
    }
    Ok(confirmed_opcodes)
}