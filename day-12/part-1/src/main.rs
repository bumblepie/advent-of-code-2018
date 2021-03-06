#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::iter::Iterator;

use regex::Regex;

fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let state_number = args[2].parse::<usize>()?;
    let file = File::open(filename)?;
    let mut lines = BufReader::new(file).lines();
    let first_line = lines.next().unwrap().expect("Input too short");
    let initial_state = read_initial_state_from_line(&first_line)?;
    //Read empty line
    lines.next();
    let rules = read_rules_from_lines(lines.filter_map(Result::ok));

    let plant_states = PlantState {
        rules,
        state: initial_state,
        same: false,
    };

    let final_state = plant_states
        .take(state_number)
        .last().unwrap();
        let sum: i32 = final_state.iter().sum();
        println!("{}", sum);
    Ok(())
}

type Pots = (bool, bool, bool, bool, bool);

fn read_initial_state_from_line(line: &str) -> Result<HashSet<i32>, String> {
    lazy_static! {
        static ref initial_state_regex: Regex = Regex::new(r"^initial state: ([#.]+)$").unwrap();
    }
    let captures = match initial_state_regex.captures(line) {
        Some(captures) => Ok(captures),
        None => Err("Unable to match regex for initial line"),
    }?;
    let plants_string = &captures[1];
    let result = plants_string
        .chars()
        .into_iter()
        .enumerate()
        .filter_map(|(index, plant_char)| match plant_char {
            '#' => Some(index as i32),
            '.' => None,
            _ => None,
        })
        .collect();
    Ok(result)
}

fn read_rules_from_lines<I>(lines: I) -> HashMap<Pots, bool>
where
    I: Iterator<Item = String>,
{
    lazy_static! {
        static ref rule_regex: Regex = Regex::new(r"^([#.]{5}) => ([#.])$").unwrap();
    }
    lines
        .enumerate()
        .map(|(index, line)| {
            let captures = rule_regex.captures(&line).expect(&format!(
                "Unable to match regex for rule {} at line {}",
                line,
                index + 2
            ));
            let precedent_string: &str = &captures[1];
            let precedent_vec: Vec<bool> = precedent_string
                .chars()
                .map(|c| match c {
                    '#' => true,
                    '.' => false,
                    _ => false,
                })
                .collect();
            let precedent: Pots = (
                precedent_vec[0],
                precedent_vec[1],
                precedent_vec[2],
                precedent_vec[3],
                precedent_vec[4],
            );
            let result = match &captures[2] {
                "#" => true,
                "." => false,
                _ => false,
            };
            (precedent, result)
        })
        .collect()
}

struct PlantState {
    rules: HashMap<Pots, bool>,
    state: HashSet<i32>,
    // If we get a repeat state, the state will be the same from then on
    same: bool,
}

impl Iterator for PlantState {
    type Item = HashSet<i32>;

    fn next(&mut self) -> Option<HashSet<i32>> {
        if self.same {
            return Some(self.state.clone());
        }
        let left = self.state.iter().min().unwrap();
        let right = self.state.iter().max().unwrap();
        let next_state: HashSet<i32> = (left - 2..right + 2)
            .map(|x| {
                let surrounding_plants: Vec<bool> = (x - 2..x + 3)
                    .map(|index| self.state.contains(&index))
                    .collect();
                let surrounding_plants: Pots = (
                    surrounding_plants[0],
                    surrounding_plants[1],
                    surrounding_plants[2],
                    surrounding_plants[3],
                    surrounding_plants[4],
                );
                (
                    x,
                    self.rules
                        .get(&surrounding_plants)
                        .unwrap_or(&false)
                        .clone(),
                )
            })
            .filter_map(|(index, plant)| match plant {
                true => Some(index),
                false => None,
            })
            .collect();
        if self.state == next_state {
            self.same = true;
            println!("Found repeat state");
        } else {
            self.state = next_state.clone();
        }
        Some(next_state)
    }
}

fn _print_plant_state(plants: &HashSet<i32>) {
    let left = plants.iter().min().unwrap().clone();
    let right = plants.iter().max().unwrap().clone();
    let plants_string = (left..right + 1)
        .map(|index| plants.contains(&index))
        .map(|plant_exists| match plant_exists {
            true => String::from("#"),
            false => String::from("."),
        })
        .collect::<Vec<String>>()
        .join("");
    println!("{}", plants_string);
}
