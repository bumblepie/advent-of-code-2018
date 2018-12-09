extern crate unicode_reader;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use unicode_reader::Graphemes;

fn main() -> Result<(), Box<std::error::Error>> {
    const ALPHABET: [char; 26] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];

    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename)?;
    let results = Graphemes::from(file).fold(
        ALPHABET.iter().map(|&c| (c, String::from(""))).collect(),
        |current_strings: HashMap<char, String>, new_code_point| {
            let next_char = new_code_point.unwrap().pop().unwrap();
            current_strings
                .into_iter()
                .map(|(letter, mut confirmed_string)| {
                    // Ignore values of the specified letter
                    if next_char.to_lowercase().to_string() == letter.to_string() {
                        return (letter, confirmed_string);
                    }
                    match confirmed_string.pop() {
                        Some(prev_char) => {
                            if prev_char != next_char
                                && prev_char.to_lowercase().to_string()
                                    == next_char.to_lowercase().to_string()
                            {
                                // REACT
                                (letter, confirmed_string)
                            } else {
                                (
                                    letter,
                                    confirmed_string
                                        + &prev_char.to_string()
                                        + &next_char.to_string(),
                                )
                            }
                        }
                        None => (letter, confirmed_string + &next_char.to_string()),
                    }
                })
                .collect::<HashMap<char, String>>()
        },
    );
    let result = results
        .iter()
        .min_by_key(|(_letter, string)| string.len())
        .unwrap();
    println!("{:?}", result);
    print!("Length: {}", result.1.len());
    Ok(())
}
