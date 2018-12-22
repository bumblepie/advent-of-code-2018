#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

use regex::Regex;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename).expect(&format!("Unable to open file {}", filename));
    let (line_results, errors): (Vec<_>, Vec<_>) =
        BufReader::new(file).lines().partition(Result::is_ok);
    if !errors.is_empty() {
        for error in errors.into_iter().filter_map(Result::err) {
            eprintln!("{}", error);
        }
        process::exit(1);
    }

    let (light_point_results, errors): (Vec<_>, Vec<_>) = line_results
        .into_iter()
        .filter_map(Result::ok)
        .map(read_light_point_from_line)
        .partition(Result::is_ok);
    if !errors.is_empty() {
        for error in errors.into_iter().filter_map(Result::err) {
            eprintln!("{}", error);
        }
        process::exit(1);
    }

    let mut light_points = light_point_results
        .into_iter()
        .filter_map(Result::ok)
        .collect();

    let mut exit = false;
    let mut current_second = 0;
    while !exit {
        let bounds = bounds_from_light_points(&light_points);
        println!("Second {}: {:?}", current_second, bounds);
        let mut instruction = String::new();
        io::stdin().read_line(&mut instruction).unwrap();
        match instruction.trim() {
            "goto" => {
                print!("Go to second: ");
                io::stdout().flush().unwrap();
                let mut second_input = String::new();
                io::stdin().read_line(&mut second_input).unwrap();
                let new_second_result = second_input.trim().parse::<i32>();
                match new_second_result {
                    Ok(new_second) => {
                        light_points = increment_state(light_points, new_second - current_second);
                        current_second = new_second;
                    }
                    Err(_) => {
                        println!("Could not parse {} as int", second_input.trim());
                    }
                }
            },

            "display" => display_state(&light_points),
            "debug" => println!("{:?}", light_points),
            "exit" => exit = true,
            _ => println!("Valid commands: goto, display, exit, debug"),
        }
    }
}

#[derive(Debug, Clone)]
struct LightPoint {
    velocity: Velocity,
    position: Position,
}

#[derive(Debug, Clone)]
struct Velocity {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Bounds {
    top: i32,
    left: i32,
    bottom: i32,
    right: i32,
}

fn read_light_point_from_line(line: String) -> Result<LightPoint, Box<Error>> {
    lazy_static! {
        static ref light_point_regex: Regex =
            Regex::new(r"position=<\s*(-?\d+),\s*(-?\d+)>\s*velocity=<\s*(-?\d+),\s*(-?\d+)>")
                .unwrap();
    }

    let captures = match light_point_regex.captures(&line) {
        Some(captures) => Ok(captures),
        None => Err(format!("Could not match regex for line {}", line)),
    }?;
    Ok(LightPoint {
        position: Position {
            x: captures[1].parse::<i32>()?,
            y: captures[2].parse::<i32>()?,
        },
        velocity: Velocity {
            x: captures[3].parse::<i32>()?,
            y: captures[4].parse::<i32>()?,
        },
    })
}

fn display_state(light_points: &Vec<LightPoint>) {
    let bounds = bounds_from_light_points(&light_points);
    let mut light_points_sorted = light_points.clone();
    light_points_sorted.sort_by(|light_point_1, light_point_2| {
        if light_point_1.position.y != light_point_2.position.y {
            return light_point_1.position.y.cmp(&light_point_2.position.y);
        }
        return light_point_1.position.x.cmp(&light_point_2.position.x);
    });

    let mut x = bounds.left;
    let mut y = bounds.top;
    for light_point in light_points_sorted {
        while y < light_point.position.y {
            // Complete the line
            print_dots(bounds.right - x + 1);
            print!("\n");
            x = bounds.left;
            y += 1;
        }
        // Do dots up to the point
        print_dots(light_point.position.x - x);

        // Do the point if it hasn't already been done
        if x <= light_point.position.x {
            print!("#");
        }
        x = light_point.position.x + 1;
    }
    //Do final dots
    while y < bounds.bottom {
        print_dots(bounds.right - x + 1);
        print!("\n");
        x = bounds.left;
        y += 1;
    }
    print_dots(bounds.right - x + 1);
    print!("\n");
}

fn print_dots(amount: i32) {
    print!("{}", (0..amount).map(|_| ".").collect::<String>());
}

fn increment_state(light_points: Vec<LightPoint>, amount: i32) -> Vec<LightPoint> {
    light_points
        .into_iter()
        .map(|light_point| LightPoint {
            position: Position {
                x: light_point.position.x + amount * light_point.velocity.x,
                y: light_point.position.y + amount * light_point.velocity.y,
            },
            ..light_point
        })
        .collect()
}

fn bounds_from_light_points(light_points: &Vec<LightPoint>) -> Bounds {
    light_points.iter().fold(
        Bounds {
            top: std::i32::MAX,
            left: std::i32::MAX,
            bottom: std::i32::MIN,
            right: std::i32::MIN,
        },
        |bounds, light_point| Bounds {
            top: i32::min(bounds.top, light_point.position.y),
            left: i32::min(bounds.left, light_point.position.x),
            bottom: i32::max(bounds.bottom, light_point.position.y),
            right: i32::max(bounds.right, light_point.position.x),
        },
    )
}
