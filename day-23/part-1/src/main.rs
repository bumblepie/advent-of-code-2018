extern crate regex;
#[macro_use]
extern crate lazy_static;

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
    let (lines, errs): (Vec<_>, Vec<_>) = BufReader::new(file).lines().partition(Result::is_ok);
    if !errs.is_empty() {
        for err in errs.into_iter().filter_map(Result::err) {
            eprintln!("{}", err);
        }
        process::exit(1);
    }
    let (nanobots, errs): (Vec<_>, Vec<_>) = lines
        .into_iter()
        .filter_map(Result::ok)
        .map(read_nanobot_from_line)
        .partition(Result::is_ok);
    if !errs.is_empty() {
        for err in errs.into_iter().filter_map(Result::err) {
            eprintln!("{}", err);
        }
        process::exit(1);
    }
    let nanobots: Vec<Nanobot> = nanobots.into_iter().filter_map(Result::ok).collect();
    let biggest_nanobot = nanobots
        .iter()
        .max_by_key(|nanobot| nanobot.radius)
        .unwrap();
    println!(
        "{}",
        nanobots
            .iter()
            .filter(|nanobot| nanobot.distance_to(biggest_nanobot) <= biggest_nanobot.radius)
            .count()
    )
}

#[derive(Debug)]
struct Nanobot {
    position: (i32, i32, i32),
    radius: i32,
}

impl Nanobot {
    fn distance_to(&self, other: &Nanobot) -> i32 {
        let x = i32::abs(self.position.0 - other.position.0);
        let y = i32::abs(self.position.1 - other.position.1);
        let z = i32::abs(self.position.2 - other.position.2);
        x + y + z
    }
}

fn read_nanobot_from_line(line: String) -> Result<Nanobot, String> {
    lazy_static! {
        static ref nanobot_regex: Regex = Regex::new(r"pos=<(-?\d+),(-?\d+),(-?\d+)>, r=(\d+)").unwrap();
    }
    if let Some(captures) = nanobot_regex.captures(&line) {
        let x = captures[1]
            .parse::<i32>()
            .expect("Unable to parse x value of nanobot");
        let y = captures[2]
            .parse::<i32>()
            .expect("Unable to parse y value of nanobot");
        let z = captures[3]
            .parse::<i32>()
            .expect("Unable to parse z value of nanobot");
        let radius = captures[4]
            .parse::<i32>()
            .expect("Unable to parse radius value of nanobot");
        Ok(Nanobot {
            position: (x, y, z),
            radius,
        })
    } else {
        Err(String::from(format!("Unable to match regex for line {}", line)))
    }
}
