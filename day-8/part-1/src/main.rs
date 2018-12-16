use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename).expect("Could not open file");
    let mut reader = BufReader::new(file);

    let mut expected_numbers = vec![ExpectedType::Header];
    let mut metadata_entries = Vec::new();
    while !expected_numbers.is_empty() {
        match expected_numbers.pop() {
            Some(ExpectedType::Header) => {
                let num_children =
                    fetch_next_number(&mut reader).expect("Error reading number of children");
                let num_metadata_entries = fetch_next_number(&mut reader)
                    .expect("Error reading number of metadata entries");
                expected_numbers.append(&mut vec![
                    ExpectedType::MetadataEntry;
                    num_metadata_entries as usize
                ]);
                expected_numbers.append(&mut vec![ExpectedType::Header; num_children as usize]);
            }
            Some(ExpectedType::MetadataEntry) => {
                metadata_entries
                    .push(fetch_next_number(&mut reader).expect("Error reading metadata entry"));
            }
            None => {
                eprintln!("Unexpected end of stack");
                process::exit(1);
            }
        }
    }
    println!("{:?}", metadata_entries);
    println!("{}", metadata_entries.iter().sum::<i32>());
}

#[derive(Debug, Clone)]
enum ExpectedType {
    Header,
    MetadataEntry,
}

fn fetch_next_number(reader: &mut BufReader<File>) -> Result<i32, Box<Error>> {
    let mut buffer = vec![];
    // Read next number
    reader.read_until(b' ', &mut buffer)?;
    let num_string = String::from_utf8(buffer)?;
    let num = num_string.trim().parse::<i32>()?;
    Ok(num)
}
