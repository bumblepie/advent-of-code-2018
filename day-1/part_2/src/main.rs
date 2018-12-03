extern crate itertools;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::env;
use std::collections::HashSet;
use itertools::Itertools;
use itertools::FoldWhile::{Continue, Done};

fn main() {
  const INITIAL_FREQUENCY: i32 = 0;
  let args: Vec<String> = env::args().collect();
  let filename = &args[1];
  let file = File::open(filename).expect("file not found");
  let frequency_changes: Vec<i32> = BufReader::new(file).lines()
    .map(|line| line.unwrap().parse::<i32>().unwrap())
    .collect();

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
