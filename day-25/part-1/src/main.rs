use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::num::ParseIntError;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename).expect("Unable to open file");
    let (lines, errors): (Vec<_>, Vec<_>) = BufReader::new(file).lines().partition(Result::is_ok);
    if !errors.is_empty() {
        for err in errors.into_iter().filter_map(Result::err) {
            eprintln!("{}", err);
        }
        process::exit(1);
    }

    let (points, errors): (Vec<_>, Vec<_>) = lines
        .into_iter()
        .filter_map(Result::ok)
        .map(read_point_from_line)
        .partition(Result::is_ok);
    if !errors.is_empty() {
        for err in errors.into_iter().filter_map(Result::err) {
            eprintln!("{}", err);
        }
        process::exit(1);
    }
    let points: Vec<_> = points.into_iter().filter_map(Result::ok).collect();
    let (constellations, _next_id) = points.iter().fold(
        (HashMap::new(), 0),
        |(mut constellations, mut next_id): (HashMap<i32, HashSet<&Point>>, i32), new_point| {
            let linked_constellations: Vec<_> = constellations
                .iter()
                .filter(|(_id, points)| {
                    points
                        .iter()
                        .filter(|pt| new_point.distance_to(pt) <= 3)
                        .count()
                        > 0
                })
                .collect();
            if linked_constellations.is_empty() {
                let mut set = HashSet::new();
                set.insert(new_point);
                constellations.insert(next_id, set);
                next_id += 1;
            } else {
                let (mut ids, joined_constellations) = linked_constellations.into_iter().fold(
                    (Vec::new(), HashSet::new()),
                    |(mut ids, mut all_points), (next_id, next_points)| {
                        ids.push(next_id);
                        all_points = all_points.union(next_points).map(|&point| point).collect();
                        all_points.insert(new_point);
                        (ids, all_points)
                    },
                );
                let first_id = ids.pop().unwrap().clone();
                let ids: Vec<i32> = ids.into_iter().map(|id| id.clone()).collect();
                ids.into_iter().for_each(|id| {
                    constellations.remove(&id);
                });
                constellations.insert(first_id, joined_constellations);
            }
            (constellations, next_id)
        },
    );
    println!("{:?}", constellations.len());
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct Point {
    i: i32,
    j: i32,
    k: i32,
    l: i32,
}

impl Point {
    fn distance_to(&self, other: &Point) -> i32 {
        i32::abs(self.i - other.i)
            + i32::abs(self.j - other.j)
            + i32::abs(self.k - other.k)
            + i32::abs(self.l - other.l)
    }
}

fn read_point_from_line(line: String) -> Result<Point, ParseIntError> {
    let values: Vec<_> = line.split(",").map(|value| value.trim()).collect();
    println!("{:?}", values);
    Ok(Point {
        i: values[0].parse::<i32>()?,
        j: values[1].parse::<i32>()?,
        k: values[2].parse::<i32>()?,
        l: values[3].parse::<i32>()?,
    })
}
