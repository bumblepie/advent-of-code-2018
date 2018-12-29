#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

use regex::Regex;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename).expect("Could not open file");
    let (lines, errors): (Vec<_>, Vec<_>) = BufReader::new(file).lines().partition(Result::is_ok);
    if !errors.is_empty() {
        for error in errors.into_iter().filter_map(Result::err) {
            eprintln!("{}", error);
        }
        process::exit(1);
    }
    let world_map = lines
        .into_iter()
        .filter_map(Result::ok)
        .map(read_clay_from_line)
        .fold(HashMap::new(), |mut world_map, new_clay| {
            for clay_square in new_clay {
                world_map.insert(clay_square, Square::Clay);
            }
            world_map
        });
    println!("Initial");
    print_world_map(&world_map);
    let final_world_state = simulate_water(Point { x: 500, y: 0 }, &world_map);
    println!("Final");
    print_world_map(&final_world_state);
    println!("{}", part_2(&final_world_state));
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Square {
    Clay,
    RunningWater,
    PoolingWater,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn below(&self) -> Point {
        Point {
            x: self.x,
            y: self.y+1,
        }
    }

    fn left(&self) -> Point {
        Point {
            x: self.x-1,
            y: self.y,
        }
    }
    fn right(&self) -> Point {
        Point {
            x: self.x+1,
            y: self.y,
        }
    }

    fn above(&self) -> Point {
        Point {
            x: self.x,
            y: self.y-1,
        }
    }

}

fn print_world_map(map: &HashMap<Point, Square>) {
    let min = Point {
        x: map.keys().min_by_key( | point| point.x).unwrap().x,
        y: map.keys().min_by_key( | point| point.y).unwrap().y,
    };
    let max = Point {
        x: map.keys().max_by_key( | point| point.x).unwrap().x,
        y: map.keys().max_by_key( | point| point.y).unwrap().y,
    };
    for y in min.y..max.y + 1 {
        for x in min.x..max.x + 1 {
            match map.get(&Point {x, y}) {
                Some(Square::Clay) => print!("#"),
                Some(Square::RunningWater) => print!("|"),
                Some(Square::PoolingWater) => print!("~"),
                _ => print!("."),
            }
        }
        println!()
    }
}

fn read_clay_from_line(line: String) -> Vec<Point> {
    lazy_static! {
        static ref restriction_regex: Regex = Regex::new(r"([xy])=(\d+\.{0,2}\d*)").unwrap();
        static ref range_regex: Regex = Regex::new(r"(\d+)\.\.(\d+)").unwrap();
    }
    let mut clay = Vec::new();
    let restrictions: HashMap<String, std::ops::Range<i32>> = line
        .split(", ")
        .into_iter()
        .map(|restriction_string| {
            let captures = restriction_regex.captures(restriction_string).unwrap();
            let axis = captures[1].to_owned();
            let restriction_string = &captures[2];
            let clay_range = match range_regex.captures(restriction_string) {
                Some(captures) => {
                    captures[1].parse::<i32>().unwrap()..(captures[2].parse::<i32>().unwrap() + 1)
                }
                None => {
                    let num = restriction_string.parse::<i32>().unwrap();
                    num..(num + 1)
                }
            };
            (axis, clay_range)
        })
        .collect();
    for x in restrictions.get("x").unwrap_or(&(0..0)).clone() {
        for y in restrictions.get("y").unwrap_or(&(0..0)).clone() {
            clay.push(Point {x, y});
        }
    }
    clay
}

fn simulate_water(starting_point: Point, initial_state: &HashMap<Point, Square>) -> HashMap<Point, Square> {
    let mut world_state = initial_state.clone();
    let mut falling_water = vec![starting_point.clone()];
    let max_y =  initial_state.keys().max_by_key( | point| point.y).unwrap().y;

    while !falling_water.is_empty() {
        let next_waterfall = falling_water.pop().unwrap();
        let mut waterfall_bottom = next_waterfall;

        // Create waterfall until we hit a non-empty square
        while world_state.get(&waterfall_bottom.below()).is_none() && waterfall_bottom.y <= max_y {
            world_state.insert(waterfall_bottom.below(), Square::RunningWater);
            waterfall_bottom = waterfall_bottom.below();
        }

        //Found a non-empty square
        match world_state.get(&waterfall_bottom.below()) {
            Some(Square::RunningWater) => {
                // Already calculated this square
                ()
            },
            Some(Square::PoolingWater) |
            Some(Square::Clay) => {
                let mut water_rising = true;
                while water_rising {
                    let (rising, mut new_waterfalls) = scan_for_new_waterfalls(&waterfall_bottom, &mut world_state);
                    falling_water.append(&mut new_waterfalls);
                    if rising {
                        waterfall_bottom = waterfall_bottom.above();
                    } else {
                        water_rising = false;
                    }
                }

            },
            // This can only happen if we have gone beyond the max y boundary
            None => (),
        }

    }
    world_state
}

// Returns: whether the water continues to rise and any new waterfalls
fn scan_for_new_waterfalls(current_waterfall_bottom: &Point, world_state: &mut HashMap<Point, Square>) -> (bool, Vec<Point>) {
    let mut new_waterfalls = Vec::new();
    let mut left = current_waterfall_bottom.clone();
    let mut right = current_waterfall_bottom.clone();
    let mut blocked_left = false;
    let mut blocked_right = false;
    loop {
        match world_state.get(&left.below()) {
            // If there's nothing below it, then we need to make a new waterfall here
            None => {
                new_waterfalls.push(left.clone());
                break;
            },
            // If there's already running water, it's already been calculated
            Some(Square::RunningWater) => break,
            // If there's pooling water or clay under it, the water is "supported"
            Some(Square::Clay) |
            Some(Square::PoolingWater) => (),
        }
        if world_state.get(&left.left()) == Some(&Square::Clay) {
            blocked_left = true;
            break;
        }
        left = left.left();
    }

    loop {
        match world_state.get(&right.below()) {
            // If there's nothing below it, then we need to make a new waterfall here
            None => {
                new_waterfalls.push(right.clone());
                break;
            },
            // If there's already running water, it's already been calculated
            Some(Square::RunningWater) => break,
            // If there's pooling water or clay under it, the water is "supported"
            Some(Square::Clay) |
            Some(Square::PoolingWater) => (),
        }
        if world_state.get(&right.right()) == Some(&Square::Clay) {
            blocked_right = true;
            break;
        }
        right = right.right();
    }

    if blocked_left && blocked_right {
        // Blocked on both sides -> raise water level and try again
        for x in left.x..right.x+1 {
            world_state.insert(Point{x, y: current_waterfall_bottom.y}, Square::PoolingWater);
        }
        (true, new_waterfalls)
    } else {
        // We were able to fall -> add any new waterfalls
        for x in left.x..right.x+1 {
            world_state.insert(Point{x, y: current_waterfall_bottom.y}, Square::RunningWater);
        }
        (false, new_waterfalls)
    }
}

fn part_1(state: &HashMap<Point, Square>) -> usize {
    let (min_y, max_y) = state.iter()
        .filter(|(_point, water_state)| **water_state == Square::Clay)
        .fold((std::i32::MAX, std::i32::MIN), |(min_y, max_y), (point, _water_state)| {
        (i32::min(min_y, point.y), i32::max(max_y, point.y))
    });
    state.iter()
        .filter(|(point, _water_state)| point.y >= min_y && point.y <= max_y)
        .filter(|(_point, water_state)| **water_state == Square::RunningWater || **water_state == Square::PoolingWater)
        .count()
}

fn part_2(state: &HashMap<Point, Square>) -> usize {
    let (min_y, max_y) = state.iter()
        .filter(|(_point, water_state)| **water_state == Square::Clay)
        .fold((std::i32::MAX, std::i32::MIN), |(min_y, max_y), (point, _water_state)| {
            (i32::min(min_y, point.y), i32::max(max_y, point.y))
        });
    state.iter()
        .filter(|(point, _water_state)| point.y >= min_y && point.y <= max_y)
        .filter(|(_point, water_state)| **water_state == Square::PoolingWater)
        .count()
}