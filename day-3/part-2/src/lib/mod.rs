extern crate itertools;
extern crate regex;

use self::itertools::Itertools;
use self::regex::Regex;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

mod error;
use self::error::InputError;

pub fn claims_with_no_overlap(filename: &str) -> Result<Vec<i32>, InputError> {
    let rectangle_regex: Regex = Regex::new(r"#(\d+) @ (\d+),(\d+): (\d+)x(\d+)").unwrap();
    let file = File::open(filename)?;
    let claim_results: Vec<Result<Claim, InputError>> = BufReader::new(file)
        .lines()
        .map(move |line| match line {
            Ok(line_string) => extract_claim(&line_string, &rectangle_regex),
            Err(err) => Err(InputError::IO(err)),
        })
        .collect();

    let mut claims = Vec::new();
    for result in claim_results {
        match result {
            Ok(claim) => claims.push(claim),
            Err(err) => return Err(err),
        }
    }

    let claims_involved_in_overlap: HashSet<i32> = claims
        .iter()
        // Get all possible combinations of different claims
        .tuple_combinations()
        // Keep tuples representing pairs of claims that overlap
        .filter(|(c1, c2)| Claim::overlaps(c1, c2))
        // Flatten to get the claim numbers of all claims involved in overlaps
        .flat_map(|(ref c1, ref c2)| vec![c1.claim_number, c2.claim_number])
        .collect();

    Ok(claims
        .iter()
        .filter(|&claim| !claims_involved_in_overlap.contains(&claim.claim_number))
        .map(|claim| claim.claim_number)
        .collect())
}

#[derive(Debug)]
struct Claim {
    claim_number: i32,
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

impl Claim {
    fn overlaps(c1: &Claim, c2: &Claim) -> bool {
        c1.left < c2.right && c1.right > c2.left && c1.top < c2.bottom && c1.bottom > c2.top
    }
}

impl Clone for Claim {
    fn clone(&self) -> Claim {
        Claim {
            claim_number: self.claim_number,
            left: self.left,
            right: self.right,
            top: self.top,
            bottom: self.bottom,
        }
    }
}

fn extract_claim(input: &str, regex: &Regex) -> Result<Claim, InputError> {
    let regex_groups = match regex.captures(&input) {
        Some(captures) => Ok(captures),
        None => Err(InputError::RegexError(input.to_string())),
    }?;
    let claim_number = regex_groups[1].parse::<i32>()?;
    let left = regex_groups[2].parse::<i32>()?;
    let top = regex_groups[3].parse::<i32>()?;
    let width = regex_groups[4].parse::<i32>()?;
    let height = regex_groups[5].parse::<i32>()?;
    Ok(Claim {
        claim_number,
        left,
        right: left + width,
        top,
        bottom: top + height,
    })
}
