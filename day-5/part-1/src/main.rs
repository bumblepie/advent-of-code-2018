extern crate unicode_reader;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use unicode_reader::Graphemes;

fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename)?;
    let mut out_file = File::create("output.txt")?;
    let result = Graphemes::from(file).fold(
        String::from(""),
        |mut confirmed_string: String, new_code_point| {
            let next_char = new_code_point.unwrap().pop().unwrap();
//            out_file.write_all(format!("{}[{}]\n", confirmed_string, next_char).as_bytes()).unwrap();
            match confirmed_string.pop() {
                Some(prev_char) => {
                    if prev_char != next_char
                        && prev_char.to_lowercase().to_string()
                            == next_char.to_lowercase().to_string()
                    {
                        // REACT
                        confirmed_string
                    } else {
                        confirmed_string + &prev_char.to_string() + &next_char.to_string()
                    }
                }
                None => confirmed_string + &next_char.to_string(),
            }
        },
    );
    println!("{}", result);
    print!("Length: {}", result.len());
    Ok(())
}
