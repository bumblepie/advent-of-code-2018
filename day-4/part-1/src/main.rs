extern crate chrono;
extern crate lazy_static;
extern crate regex;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

mod lib;

fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    // Open file, read lines
    let file = File::open(filename).unwrap_or_else(|_err| {
        eprintln!("Unable to open file {}", filename);
        process::exit(1);
    });
    let (lines, errors): (Vec<_>, Vec<_>) = BufReader::new(file).lines().partition(Result::is_ok);
    if !errors.is_empty() {
        for error in errors {
            eprintln!("{}", error.unwrap_err());
        }
        process::exit(1);
    }

    // Parse the lines into shift events
    let (shift_event_results, errors): (Vec<_>, Vec<_>) = lines
        .into_iter()
        .map(Result::unwrap)
        .map(|line| lib::get_event_from_line(&line))
        .partition(Result::is_ok);
    if !errors.is_empty() {
        for error in errors {
            eprintln!("{}", error.unwrap_err());
        }
        process::exit(1);
    }

    let shift_events: Vec<lib::ShiftEvent> = shift_event_results
        .into_iter()
        .map(Result::unwrap)
        .collect();

    let sleeps = lib::get_sleeps_from_shift_events(shift_events);

    let guard_sleep_totals = lib::get_guard_sleep_totals(&sleeps);

    let (most_asleep_guard, _minutes_slept) = guard_sleep_totals
        .into_iter()
        .max_by_key(|(_guard_number, duration)| duration.num_seconds())
        .unwrap_or((0, chrono::Duration::seconds(0)));

    let sleeps_by_lazy_guard: Vec<lib::Sleep> =
        lib::get_sleeps_for_guard(sleeps, most_asleep_guard);

    let time_slept_at_minute = lib::get_time_slept_at_minute(&sleeps_by_lazy_guard);

    let (most_slept_minute, time_slept) = time_slept_at_minute
        .into_iter()
        .max_by_key(|(_minute, total)| *total)
        .unwrap();
    println!(
        "Guard {} is the asleep at minute {} the most ({} minutes slept)",
        most_asleep_guard, most_slept_minute, time_slept
    );
    println!("Answer: {}", most_asleep_guard * (most_slept_minute as i32));
    Ok(())
}
