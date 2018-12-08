extern crate regex;

use self::regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

mod error;
use self::error::InputError;

pub fn double_claimed_area(filename: &str) -> Result<usize, InputError> {
    let rectangle_regex: Regex = Regex::new(r"#\d+ @ (\d+),(\d+): (\d+)x(\d+)").unwrap();

    let file = File::open(filename)?;
    let fabric_claims = BufReader::new(file)
        .lines()
        .map(|line| match line {
            Ok(line_string) => extract_rectangle(line_string, &rectangle_regex),
            Err(err) => Err(InputError::IO(err)),
        })
        .fold(
            Ok(HashMap::new()),
            |fabric_claims_result: Result<HashMap<(i32, i32), i32>, InputError>,
             rectangle_result| {
                // Return first occurring error before a new one from rectangle_result
                let mut fabric_claims = fabric_claims_result?;
                let rect: Rectangle = rectangle_result?;

                for x in rect.left..rect.right {
                    for y in rect.top..rect.bottom {
                        let num_claims = fabric_claims.get(&(x, y)).unwrap_or(&0) + 1;
                        fabric_claims.insert((x, y), num_claims);
                    }
                }
                Ok(fabric_claims)
            },
        )?;
    Ok(fabric_claims
        .iter()
        .filter(|&(_point, &num_claims)| num_claims > 1)
        .count())
}

#[derive(Debug)]
struct Rectangle {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

fn extract_rectangle(input: String, regex: &Regex) -> Result<Rectangle, InputError> {
    let regex_groups = match regex.captures(&input) {
        Some(captures) => Ok(captures),
        None => Err(InputError::RegexError(input.clone())),
    }?;
    let left = regex_groups[1].parse::<i32>()?;
    let top = regex_groups[2].parse::<i32>()?;
    let width = regex_groups[3].parse::<i32>()?;
    let height = regex_groups[4].parse::<i32>()?;
    Ok(Rectangle {
        left,
        right: left + width,
        top,
        bottom: top + height,
    })
}
