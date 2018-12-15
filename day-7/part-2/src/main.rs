extern crate regex;
#[macro_use]
extern crate lazy_static;

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::iter::FromIterator;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let workers = args[2]
        .parse::<usize>()
        .expect("Second argument must be an positive integer");
    if workers < 1 {
        eprintln!("Must have more than zero workers to complete instructions");
        process::exit(1);
    }
    let file = File::open(filename).expect("File not found");
    let (line_results, errs): (Vec<_>, Vec<_>) =
        BufReader::new(file).lines().partition(Result::is_ok);
    if !errs.is_empty() {
        for err in errs.into_iter().filter_map(Result::err) {
            eprintln!("{}", err);
        }
        process::exit(1);
    }
    let lines: Vec<String> = line_results.into_iter().filter_map(Result::ok).collect();
    let (requirement_results, errs): (Vec<_>, Vec<_>) = lines
        .into_iter()
        .map(get_requirement_from_line)
        .partition(Result::is_ok);
    if !errs.is_empty() {
        for err in errs.into_iter().filter_map(Result::err) {
            eprintln!("{}", err);
        }
        process::exit(1);
    }
    let mut to_do_steps = requirement_results.into_iter().filter_map(Result::ok).fold(
        HashMap::new(),
        |mut steps: HashMap<String, HashSet<String>>, requirement: Requirement| {
            // Create new set for both steps if needed
            steps
                .entry(requirement.target_identifier.clone())
                .or_insert(HashSet::new());
            steps
                .entry(requirement.required_identifier.clone())
                .or_insert(HashSet::new());

            //Add requirement
            steps
                .get_mut(&requirement.target_identifier)
                .unwrap()
                .insert(requirement.required_identifier);
            steps
        },
    );
    for step in to_do_steps.iter() {
        println!("{:?}", step);
    }

    let mut completed_steps: Vec<String> = Vec::new();
    let mut working_steps: Vec<StepInProgress> = Vec::new();
    let mut seconds_passed = 0;
    while !to_do_steps.is_empty() {
        println!("Assigning free workers");
        // Assign free workers
        let mut next_step = get_next_step(&to_do_steps, &completed_steps);
        while working_steps.len() < workers && next_step.is_some() {
            let step = next_step.unwrap();
            working_steps.push(StepInProgress {
                step_identifier: step.to_string(),
                time_til_complete: get_time_til_complete(&step.to_lowercase()),
            });
            to_do_steps.remove(&step);
            next_step = get_next_step(&to_do_steps, &completed_steps);
        }
        println!("{:?}", working_steps);

        // Complete next step
        let next_completed_index = match working_steps
            .iter()
            .enumerate()
            .min_by_key(|(_index, step)| step.time_til_complete)
        {
            Some((index, _completed_step)) => Some(index),
            None => None,
        };
        match next_completed_index {
            Some(index) => {
                // Move step to completed
                let completed_step = working_steps.remove(index);
                println!("Completed {:?}", completed_step);
                completed_steps.push(completed_step.step_identifier.to_string());
                // Reduce time to complete on other steps
                seconds_passed += completed_step.time_til_complete;
                for step in &mut working_steps {
                    step.time_til_complete -= completed_step.time_til_complete;
                }
            }
            None => (),
        }
    }
    println!("{}", seconds_passed);
}

#[derive(Debug)]
struct Requirement {
    required_identifier: String,
    target_identifier: String,
}

#[derive(Debug)]
struct StepInProgress {
    step_identifier: String,
    time_til_complete: i32,
}

fn get_requirement_from_line(line: String) -> Result<Requirement, String> {
    lazy_static! {
        static ref requirement_regex: Regex = Regex::new(
            r"Step ([[:alpha:]]+) must be finished before step ([[:alpha:]]+) can begin."
        )
        .unwrap();
    }
    let captures = match requirement_regex.captures(&line) {
        Some(captures) => Ok(captures),
        None => Err(format!("Could not read regex {}", line)),
    }?;
    let required_identifier = String::from(&captures[1]);
    let target_identifier = String::from(&captures[2]);
    Ok(Requirement {
        required_identifier,
        target_identifier,
    })
}

fn get_next_step(
    steps: &HashMap<String, HashSet<String>>,
    completed_steps: &Vec<String>,
) -> Option<String> {
    let completed_steps_set: HashSet<&String> = HashSet::from_iter(completed_steps.iter());
    let mut valid_steps: Vec<&String> = steps
        .iter()
        .filter(|(_step_id, requirements)| {
            let requirement_refs_set = HashSet::from_iter(requirements.iter());
            requirement_refs_set.is_subset(&completed_steps_set)
        })
        .map(|(step_id, _requirements)| step_id)
        .collect();
    valid_steps.sort();
    match valid_steps.first() {
        Some(step) => Some(step.to_string()),
        None => None,
    }
}

fn get_time_til_complete(step_identifier: &str) -> i32 {
    lazy_static! {
        static ref alphabet: String = String::from("abcdefghijklmnopqrstuvwxyz");
    }
    let extra_time: i32 = step_identifier
        .to_lowercase()
        .chars()
        .map(|c| alphabet.find(c).unwrap() as i32)
        .sum();
    // Add 61 to make A = 61, B = 62...
    61 + extra_time
}
