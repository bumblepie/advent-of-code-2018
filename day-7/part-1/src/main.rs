extern crate regex;
#[macro_use]
extern crate lazy_static;

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::iter::FromIterator;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
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
    let mut steps = requirement_results.into_iter().filter_map(Result::ok).fold(
        HashMap::new(),
        |mut steps: HashMap<String, HashSet<String>>, requirement: Requirement| {
            // Create new set for both steps if needed
            steps.entry(requirement.target_identifier.clone()).or_insert(HashSet::new());
            steps.entry(requirement.required_identifier.clone()).or_insert(HashSet::new());

            //Add requirement
            steps
                .get_mut(&requirement.target_identifier)
                .unwrap()
                .insert(requirement.required_identifier);
            steps
        },
    );
    for step in steps.iter() {
        println!("{:?}", step);
    }

    let mut completed_steps = Vec::new();
    while !steps.is_empty() {
        let completed_steps_set: HashSet<&String> = HashSet::from_iter(completed_steps.iter());
        let mut valid_steps: Vec<&String> = steps.iter().filter(|(_step_id, requirements)| {
            let requirement_refs_set = HashSet::from_iter(requirements.iter());
            requirement_refs_set.is_subset(&completed_steps_set)
        }).map(|(step_id, _requirements)| step_id).collect();
        valid_steps.sort();
        match valid_steps.first() {
            Some(step) => {
                completed_steps.push(step.to_string());
                steps.remove(&step.to_string());
            },
            None => {
                eprintln!("Could not complete instructions - perhaps there is a cycle");
                process::exit(1);
            }
        }
    }
    println!("{}", completed_steps.join(""));
}

struct Requirement {
    required_identifier: String,
    target_identifier: String,
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
        None => Err("Could not read regex"),
    }?;
    let required_identifier = String::from(&captures[1]);
    let target_identifier = String::from(&captures[2]);
    Ok(Requirement {
        required_identifier,
        target_identifier,
    })
}
