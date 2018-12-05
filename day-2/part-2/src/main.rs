use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename).expect("file not found");
    let words: Vec<String> = BufReader::new(file).lines()
      .map(|line| line.unwrap())
      .collect();

  let result = find_close_words(words);

  println!("result: {:?}", result);
}

fn find_close_words (words: Vec<String>) -> Option<String>{
  for i in 0..words.len() {
    for j in i..words.len() {
      // Assumption: words are always the same length
      if words[i] == words[j] || words[i].len() != words[j].len() {
        continue;
      }
      let mut diff_count = 0;
      let mut word1_chars = words[i].chars();
      let mut word2_chars = words[j].chars();
      let mut resulting_word = String::new();

      while diff_count < 2 {
        let word1_char = word1_chars.next();
        let word2_char = word2_chars.next();

        if word1_char.is_some() && word2_char.is_some() {
          if word1_char.unwrap() != word2_char.unwrap() {
            diff_count += 1;

          } else {
            resulting_word.push(word1_char.unwrap());
          }
        } else {
          break;
        }
      }

      if diff_count == 1 {
        return Some(resulting_word);
      }
    }
  }
  None
}
