extern crate regex;
#[macro_use]
extern crate lazy_static;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

use regex::Regex;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename).expect("Unable to open file");
    let (lines, errs): (Vec<_>, Vec<_>) = BufReader::new(file).lines().partition(Result::is_ok);
    if !errs.is_empty() {
        for err in errs.into_iter().filter_map(Result::err) {
            eprintln!("{}", err);
        }
        process::exit(1);
    }
    let (nanobots, errs): (Vec<_>, Vec<_>) = lines
        .into_iter()
        .filter_map(Result::ok)
        .map(read_nanobot_from_line)
        .partition(Result::is_ok);
    if !errs.is_empty() {
        for err in errs.into_iter().filter_map(Result::err) {
            eprintln!("{}", err);
        }
        process::exit(1);
    }
    let nanobots: Vec<Nanobot> = nanobots.into_iter().filter_map(Result::ok).collect();
//    let min_radius = nanobots.iter().min_by_key(|nanobot| nanobot.radius).unwrap().radius;
    let result = part_2(&nanobots);
    println!("{:?}", result);
}
fn part_2(nanobots: &Vec<Nanobot>) -> i32 {
    let bounds = nanobots.iter().fold((std::i32::MAX, std::i32::MIN, std::i32::MAX, std::i32::MIN, std::i32::MAX, std::i32::MIN), |mut bounds, nanobot| {
        bounds.0 = i32::min(bounds.0, nanobot.position.0);
        bounds.1 = i32::max(bounds.1, nanobot.position.0);
        bounds.2 = i32::min(bounds.2, nanobot.position.1);
        bounds.3 = i32::max(bounds.3, nanobot.position.1);
        bounds.4 = i32::min(bounds.4, nanobot.position.2);
        bounds.5 = i32::max(bounds.5, nanobot.position.2);
        bounds
    });

    let x_length = bounds.1 - bounds.0;
    let y_length = bounds.3 - bounds.2;
    let z_length = bounds.5 - bounds.4;

    println!("{:?}", bounds);
//    let mut box_size = min_radius;
    let initial_search_cube = SearchCube {
        centre_point: (
            bounds.0 + x_length/2,
            bounds.2 + y_length/2,
            bounds.4 + z_length/2,
        ),
        side_length: i32::max(x_length, i32::max(y_length, z_length)),
    };

    // Take advantage of Rust's automatic tuple sorting
    // First sort by number of intersecting bots to maximise that
    // Then search cubes are sorted by their negative manhattan distance from the origin,
    // so closer points are "greater" than ones further away
    let mut search_regions = BinaryHeap::new();
    search_regions.push((nanobots.len(), initial_search_cube));

    loop {
        println!("FINDING NEXT CUBE");
        for s in search_regions.iter() {
            println!("{:?}", s);
        }
        let (num_bots, next_cube) = search_regions.pop().unwrap();
        println!("CHOSEN_CUBE:\n{}, {:?}\n\n", num_bots, next_cube);
        match next_cube.subdivide() {
            Some(smaller_cubes) => {
                smaller_cubes.into_iter()
                    .for_each(|cube|
                        search_regions.push((cube.get_intersecting_nanobots(&nanobots), cube))
                    );
            },
            None => return distance_to(&next_cube.centre_point, &(0,0,0)),
        }
    }
}


#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Nanobot {
    position: (i32, i32, i32),
    radius: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct SearchCube {
    centre_point: (i32, i32, i32),
    side_length: i32,
}

impl SearchCube {
    fn subdivide(&self) -> Option<Vec<SearchCube>> {
        if self.side_length > 1 {
            let side_length = self.side_length / 2;
            let mut result = Vec::new();
            for x_index in 0..2 {
                for y_index in 0..2 {
                    for z_index in 0..2 {
                        let x = self.centre_point.0 + (x_index * side_length) - (side_length / 2);
                        let y = self.centre_point.1 + (y_index * side_length) - (side_length / 2);
                        let z = self.centre_point.2 + (z_index * side_length) - (side_length / 2);
                        result.push(SearchCube {
                            centre_point: (x, y, z),
                            side_length,
                        })
                    }
                }
            }
            Some(result)
        } else {
            None
        }
    }

    fn get_intersecting_nanobots(&self, nanobots: &Vec<Nanobot>) -> usize {
        nanobots.iter()
            .filter(|nanobot|
                distance_to(&nanobot.position, &self.centre_point) <= (nanobot.radius + ((self.side_length / 2) * 3 ))
            ).count()
    }
}

impl Ord for SearchCube {
    fn cmp(&self, other: &SearchCube) -> Ordering {
        distance_to(&self.centre_point, &(0,0,0)).cmp(&distance_to(&other.centre_point, &(0,0,0))).reverse()
    }
}

impl PartialOrd for SearchCube {
    fn partial_cmp(&self, other: &SearchCube) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn distance_to(pos: &(i32, i32, i32), other: &(i32, i32, i32)) -> i32 {
    i32::abs(pos.0 - other.0) + i32::abs(pos.1 - other.1) + i32::abs(pos.2 - other.2)
}

fn read_nanobot_from_line(line: String) -> Result<Nanobot, String> {
    lazy_static! {
        static ref nanobot_regex: Regex =
            Regex::new(r"pos=<(-?\d+),(-?\d+),(-?\d+)>, r=(\d+)").unwrap();
    }
    if let Some(captures) = nanobot_regex.captures(&line) {
        let x = captures[1]
            .parse::<i32>()
            .expect("Unable to parse x value of nanobot");
        let y = captures[2]
            .parse::<i32>()
            .expect("Unable to parse y value of nanobot");
        let z = captures[3]
            .parse::<i32>()
            .expect("Unable to parse z value of nanobot");
        let radius = captures[4]
            .parse::<i32>()
            .expect("Unable to parse radius value of nanobot");
        Ok(Nanobot {
            position: (x, y, z),
            radius,
        })
    } else {
        Err(String::from(format!(
            "Unable to match regex for line {}",
            line
        )))
    }
}
