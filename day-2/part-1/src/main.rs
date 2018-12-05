use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename).expect("file not found");
    let result = BufReader::new(file).lines()
      // get a count of each character in each line
      .map(|line| line.unwrap().chars()
        .fold(HashMap::new(), |mut char_counts: HashMap<char, i32>, new_char| {
            let count = char_counts.get(&new_char).unwrap_or(&0).to_owned() + 1;
            char_counts.insert(new_char, count);
            char_counts
        }))
      // Get if the line has a character that appears exactly 2 or 3 times
      .map(|char_count| (char_count.values().find(|&&x| x == 2).is_some(), char_count.values().find(|&&x| x == 3).is_some()))
      .fold((0, 0), | (mut contains_two_count, mut contains_three_count), (contains_two, contains_three)| {
        if contains_two {
          contains_two_count +=1;
        }
        if contains_three {
          contains_three_count +=1;
        }
        (contains_two_count, contains_three_count)
      });
      // Count the number of lines which have a max char count needed for the checksum operation
    let checksum = result.0 * result.1;
    println!("Checksum: {}", checksum);
}
