use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::env;

fn main() {
  let args: Vec<String> = env::args().collect();
  let filename = &args[1];
  let file = File::open(filename).expect("file not found");
  let result = BufReader::new(file).lines()
    .map(|line| line.unwrap().parse::<i32>().unwrap())
    .fold(0, |sum, value| sum + value);
    println!("{}", result);
}
