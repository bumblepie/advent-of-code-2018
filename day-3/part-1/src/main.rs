use std::env;
use std::process;

mod lib;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please provide a filename");
        process::exit(1);
    }
    let filename = args[1].clone();
    let result = lib::double_claimed_area(&filename);
    match result {
        Ok(area) => {
            println!("Area with multipl claims: {} square inches", area);
        }
        Err(err) => {
            eprintln!("Error parsing input: {}", err);
            process::exit(1);
        }
    }
}
