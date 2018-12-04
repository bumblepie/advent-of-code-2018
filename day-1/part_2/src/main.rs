extern crate itertools;

use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use std::num;
use std::process;

use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;

fn main() {
  const INITIAL_FREQUENCY: i32 = 0;
  let args: Vec<String> = env::args().collect();
  let filename = &args[1];
  let file = File::open(filename).expect("file not found");
  let frequency_changes_result: Result<Vec<i32>, BadInputError> = BufReader::new(file).lines()
    .enumerate()
    .map(read_int_from_line)
    .collect();

  if frequency_changes_result.is_err() {
    eprintln!("Error parsing input:\n{}", frequency_changes_result.unwrap_err());
    process::exit(1);
  }
  let frequency_changes = frequency_changes_result.unwrap();

  let result = frequency_changes.iter().cycle()
    .fold_while((INITIAL_FREQUENCY, HashSet::new()), | (frequency, mut frequencies_seen), change| {
      let new_frequency = frequency + change;
      if frequencies_seen.contains(&new_frequency) {
        Done((new_frequency, frequencies_seen))
      } else {
        frequencies_seen.insert(new_frequency);
        Continue((new_frequency, frequencies_seen))
      }
    }).into_inner();

  let frequency = result.0;
  println!("Seen again: {}", frequency);
}

fn read_int_from_line((line_index, line_result): (usize, Result<String, io::Error>)) -> Result<i32, BadInputError> {
  let line_string = line_result.map_err(|e: io::Error| -> BadInputError {
    BadInputError {
      line_number: line_index + 1,
      line: None,
      cause: Some(BadInputErrorCause::Io(e)),
    }
  })?;
  line_string.parse::<i32>().map_err(|e: num::ParseIntError| -> BadInputError {
    BadInputError {
      line_number: line_index + 1,
      line: Some(line_string),
      cause: Some(BadInputErrorCause::Parse(e)),
    }
  })
}

#[derive(Debug)]
enum BadInputErrorCause {
  Io(io::Error),
  Parse(num::ParseIntError),
}

impl fmt::Display for BadInputErrorCause {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      BadInputErrorCause::Io(ref e) => e.fmt(f),
      BadInputErrorCause::Parse(ref e) => e.fmt(f),
    }
  }
}

impl Error for BadInputErrorCause {
  fn cause(&self) -> Option<&Error> {
    match *self {
      BadInputErrorCause::Io(ref e) => Some(e),
      BadInputErrorCause::Parse(ref e) => Some(e),
    }
  }
}

#[derive(Debug)]
struct BadInputError {
  line_number: usize,
  line: Option<String>,
  cause: Option<BadInputErrorCause>,
}

impl fmt::Display for BadInputError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.cause {
      Some(BadInputErrorCause::Io(_)) => write!(f, "Error reading input file at line {}", self.line_number),
      Some(BadInputErrorCause::Parse(_)) => write!(f,
                                                   "Bad input at line {}: \"{}\"",
                                                   self.line_number,
                                                   self.line.clone().unwrap_or("???".to_string())),
      None => write!(f, "Unknown issue at line {}", self.line_number),
    }
  }
}

impl Error for BadInputError {
  fn cause(&self) -> Option<&Error> {
    match self.cause {
      Some(ref e) => Some(e),
      None => None,
    }
  }
}
