extern crate regex;
#[macro_use]
extern crate lazy_static;

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use regex::Regex;

#[derive(Debug, Eq, PartialEq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    const TARGET_REGION_SIZE: i32 = 10000;

    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename).expect("Unable to open file");
    let (lines, _errors): (Vec<_>, Vec<_>) = BufReader::new(file).lines().partition(Result::is_ok);
    let (point_results, _errors): (Vec<_>, Vec<_>) = lines
        .into_iter()
        .filter_map(Result::ok)
        .map(read_point_from_line)
        .partition(Result::is_ok);
    let points: HashMap<usize, Point> = point_results
        .into_iter()
        .filter_map(Result::ok)
        .enumerate()
        .collect();
    let bounds = points.iter().fold(
        (
            Point {
                x: <i32>::max_value(),
                y: <i32>::max_value(),
            },
            Point {
                x: <i32>::min_value(),
                y: <i32>::min_value(),
            },
        ),
        |bounds, (_id, point)| {
            let min_point = Point {
                x: i32::min(bounds.0.x, point.x),
                y: i32::min(bounds.0.y, point.y),
            };
            let max_point = Point {
                x: i32::max(bounds.1.x, point.x),
                y: i32::max(bounds.1.y, point.y),
            };
            (min_point, max_point)
        },
    );

    let mut region_size = 0;
    // Calculate region size
    for x in bounds.0.x..bounds.1.x {
        for y in bounds.0.y..bounds.1.y {
            let current_point = Point { x, y };
            let sum_of_distances = points.iter().fold(0, |sum_of_distances, (_id, point)| {
                sum_of_distances + manhattan_distance(point, &current_point)
            });
            if sum_of_distances < TARGET_REGION_SIZE {
                region_size += 1;
            }
        }
    }

    //Calculate total areas for finite areas, then find max
    println!("Region size: {}", region_size);
}

fn read_point_from_line(line: String) -> Result<Point, Box<std::error::Error>> {
    lazy_static! {
        static ref point_regex: Regex = Regex::new(r"(\d+), (\d+)").unwrap();
    }
    let captures = match point_regex.captures(&line) {
        None => Err("Couldn't match regex"),
        Some(captures) => Ok(captures),
    }?;
    let x = captures[1].parse::<i32>()?;
    let y = captures[2].parse::<i32>()?;
    Ok(Point { x, y })
}

fn closest_point_to_point<'a>(
    target_point: &Point,
    points: &'a HashMap<usize, Point>,
) -> Option<&'a usize> {
    let (_min, pts_with_min_manhattan_distance) = points.iter().fold(
        (<i32>::max_value(), Vec::new()),
        |(current_min, mut points), (id, next_point)| {
            let distance = manhattan_distance(next_point, target_point);
            match distance {
                distance if distance < current_min => (distance, vec![id]),
                _distance if distance == current_min => {
                    points.push(id);
                    (current_min, points)
                }
                _distance => (current_min, points),
            }
        },
    );
    match pts_with_min_manhattan_distance.len() {
        0 => None,
        1 => Some(*pts_with_min_manhattan_distance.first().unwrap()),
        _ => None,
    }
}

fn manhattan_distance(p1: &Point, p2: &Point) -> i32 {
    i32::abs(p1.x - p2.x) + i32::abs(p1.y - p2.y)
}

fn total_area(point_and_id: (&usize, &Point), all_points: &HashMap<usize, Point>) -> i32 {
    let (id, target_point) = point_and_id;
    let mut count = 0;
    let mut points_checked = HashSet::new();
    let mut points_to_check: Vec<Point> = Vec::new();
    points_to_check.push(Point {
        x: target_point.x,
        y: target_point.y,
    });

    while !points_to_check.is_empty() {
        let current_point = points_to_check.pop().unwrap();
        if closest_point_to_point(&current_point, all_points) == Some(id) {
            count += 1;

            get_adjacent_points(&current_point)
                .into_iter()
                .for_each(|adj_point| {
                    if !points_checked.contains(&adj_point) && !points_to_check.contains(&adj_point)
                    {
                        points_to_check.push(adj_point);
                    }
                });
        }
        points_checked.insert(current_point);
    }
    count
}

fn get_adjacent_points(point: &Point) -> Vec<Point> {
    vec![
        Point {
            x: point.x - 1,
            ..*point
        },
        Point {
            x: point.x + 1,
            ..*point
        },
        Point {
            y: point.y - 1,
            ..*point
        },
        Point {
            y: point.y + 1,
            ..*point
        },
    ]
}
