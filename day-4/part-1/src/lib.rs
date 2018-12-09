use lazy_static::*;

use chrono::prelude::*;
use regex::Regex;
use std::cmp::Ordering;

use chrono::Duration;
use std::collections::HashMap;

pub fn get_event_from_line(line: &str) -> Result<ShiftEvent, Box<std::error::Error>> {
    lazy_static! {
        static ref event_regex: Regex =
            Regex::new(r"\[(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})\] (.*)").unwrap();
        static ref guard_shift_regex: Regex = Regex::new(r"Guard #(\d+) begins shift").unwrap();
    }

    let regex_groups = match event_regex.captures(line) {
        None => Err("Could not match full event regex"),
        Some(captures) => Ok(captures),
    }?;
    let year = regex_groups[1].parse::<i32>()?;
    let month = regex_groups[2].parse::<u32>()?;
    let day = regex_groups[3].parse::<u32>()?;
    let hour = regex_groups[4].parse::<u32>()?;
    let minute = regex_groups[5].parse::<u32>()?;

    let datetime = Local.ymd(year, month, day).and_hms(hour, minute, 0);

    let event_string = &regex_groups[6];

    let event_type = match event_string {
        "falls asleep" => Ok(EventType::FallAsleep),
        "wakes up" => Ok(EventType::WakeUp),
        str => match guard_shift_regex.captures(str) {
            None => Err("Could not match guard shift regex"),
            Some(captures) => {
                let guard_number = captures[1].parse::<i32>()?;
                Ok(EventType::ShiftStart { guard_number })
            }
        },
    }?;

    Ok(ShiftEvent {
        datetime,
        event_type,
    })
}

pub fn get_sleeps_from_shift_events(mut shift_events: Vec<ShiftEvent>) -> Vec<Sleep> {
    // Sort shift events to ensure they are in order
    shift_events.sort();

    let (sleeps, _end_state) = shift_events.into_iter().fold(
        (
            Vec::new(),
            GuardState {
                guard_number: None,
                asleep_since: None,
            },
        ),
        |(mut sleeps, state), guard_shift_event| {
            let new_state = match guard_shift_event.event_type {
                EventType::ShiftStart { guard_number } => GuardState {
                    guard_number: Some(guard_number),
                    asleep_since: None,
                },
                EventType::FallAsleep => GuardState {
                    asleep_since: Some(guard_shift_event.datetime),
                    ..state
                },
                EventType::WakeUp => {
                    if state.guard_number.is_some() && state.asleep_since.is_some() {
                        sleeps.push(Sleep {
                            guard_number: state.guard_number.unwrap(),
                            asleep_between: (
                                state.asleep_since.unwrap(),
                                guard_shift_event.datetime,
                            ),
                        })
                    } else {
                        eprintln!("Weird state found {:?}, {:?}", state, guard_shift_event);
                    }
                    GuardState {
                        asleep_since: None,
                        ..state
                    }
                }
            };
            (sleeps, new_state)
        },
    );
    sleeps
}

pub fn get_guard_sleep_totals(sleeps: &[Sleep]) -> HashMap<i32, Duration> {
    sleeps
        .iter()
        .fold(HashMap::new(), |mut guard_sleep_totals, new_sleep| {
            let default_duration = chrono::Duration::seconds(0);
            let current_sleep_duration = guard_sleep_totals
                .get(&new_sleep.guard_number)
                .unwrap_or(&default_duration);
            let new_sleep_duration = new_sleep
                .asleep_between
                .1
                .signed_duration_since(new_sleep.asleep_between.0);
            let total_sleep_duration = *current_sleep_duration + new_sleep_duration;
            guard_sleep_totals.insert(new_sleep.guard_number, total_sleep_duration);
            guard_sleep_totals
        })
}

pub fn get_sleeps_for_guard(sleeps: Vec<Sleep>, guard_number: i32) -> Vec<Sleep> {
    sleeps
        .into_iter()
        .filter(|sleep| sleep.guard_number == guard_number)
        .collect()
}

pub fn get_time_slept_at_minute(sleeps: &[Sleep]) -> HashMap<i32, i32> {
    let mut time_slept_at_minute = HashMap::new();
    for minute in 0..60 {
        let total = sleeps
            .iter()
            .filter(|sleep| {
                let start_minute = sleep.asleep_between.0.minute() as i32;
                let end_minute = sleep.asleep_between.1.minute() as i32;
                minute >= start_minute && minute < end_minute
            })
            .count() as i32;
        time_slept_at_minute.insert(minute, total);
    }
    time_slept_at_minute
}

#[derive(Debug, Eq, PartialEq)]
enum EventType {
    ShiftStart { guard_number: i32 },
    FallAsleep,
    WakeUp,
}

#[derive(Debug)]
pub struct ShiftEvent {
    datetime: DateTime<Local>,
    event_type: EventType,
}

impl Ord for ShiftEvent {
    fn cmp(&self, other: &ShiftEvent) -> Ordering {
        self.datetime.cmp(&other.datetime)
    }
}

impl PartialOrd for ShiftEvent {
    fn partial_cmp(&self, other: &ShiftEvent) -> Option<Ordering> {
        self.datetime.partial_cmp(&other.datetime)
    }
}

impl Eq for ShiftEvent {}

impl PartialEq for ShiftEvent {
    fn eq(&self, other: &ShiftEvent) -> bool {
        self.datetime == other.datetime
    }
}

#[derive(Debug)]
pub struct Sleep {
    guard_number: i32,
    asleep_between: (DateTime<Local>, DateTime<Local>),
}

#[derive(Debug)]
struct GuardState {
    guard_number: Option<i32>,
    asleep_since: Option<DateTime<Local>>,
}
