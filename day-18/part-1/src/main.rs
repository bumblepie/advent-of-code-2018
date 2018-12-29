use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

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

    let initial_state: WorldState = lines
        .into_iter()
        .filter_map(Result::ok)
        .map(read_world_line)
        .enumerate()
        .flat_map(|(y, line)| {
            line.into_iter().enumerate().map(move |(x, ground_state)| {
                (
                    Point {
                        x: x as i32,
                        y: y as i32,
                    },
                    ground_state,
                )
            })
        })
        .collect();

    let mut state = initial_state.clone();
    print_world_state(&state);
    for i in 1..11 {
        println!("After {} seconds", i);
        state = next_state(&state);
        print_world_state(&state);
    }

    let lumberyard_count = state
        .iter()
        .filter(|(_point, ground_state)| **ground_state == GroundState::LumberYard)
        .count();

    let trees_count = state
        .iter()
        .filter(|(_point, ground_state)| **ground_state == GroundState::Trees)
        .count();

    println!(
        "{} Lumberyards * {} Trees = {}",
        lumberyard_count,
        trees_count,
        lumberyard_count * trees_count
    )
}

type WorldState = HashMap<Point, GroundState>;

#[derive(Clone, Debug, Eq, PartialEq)]
enum GroundState {
    OpenGround,
    Trees,
    LumberYard,
}

impl fmt::Display for GroundState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GroundState::OpenGround => ".",
                GroundState::Trees => "|",
                GroundState::LumberYard => "#",
            }
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn adjacent(&self) -> Vec<Point> {
        vec![
            Point {
                y: self.y - 1,
                x: self.x - 1,
            },
            Point {
                y: self.y - 1,
                ..*self
            },
            Point {
                y: self.y - 1,
                x: self.x + 1,
            },
            Point {
                x: self.x - 1,
                ..*self
            },
            Point {
                x: self.x + 1,
                ..*self
            },
            Point {
                y: self.y + 1,
                x: self.x - 1,
            },
            Point {
                y: self.y + 1,
                ..*self
            },
            Point {
                y: self.y + 1,
                x: self.x + 1,
            },
        ]
    }
}

fn print_world_state(state: &WorldState) {
    let (x_min, x_max, y_min, y_max) = state.iter().fold(
        (std::i32::MAX, std::i32::MIN, std::i32::MAX, std::i32::MIN),
        |(x_min, x_max, y_min, y_max), (point, _ground_state)| {
            (
                i32::min(x_min, point.x),
                i32::max(x_max, point.x),
                i32::min(y_min, point.y),
                i32::max(y_max, point.y),
            )
        },
    );
    for y in y_min..y_max + 1 {
        println!(
            "{}",
            (x_min..x_max + 1)
                .map(|x| state
                    .get(&Point { x, y })
                    .unwrap_or(&GroundState::OpenGround))
                .map(|ground_state| ground_state.to_string())
                .collect::<Vec<String>>()
                .join("")
        );
    }
}

fn read_world_line(line: String) -> Vec<GroundState> {
    line.chars()
        .into_iter()
        .map(|character| match character {
            '.' => GroundState::OpenGround,
            '|' => GroundState::Trees,
            '#' => GroundState::LumberYard,
            _ => panic!("Unrecognised character in input"),
        })
        .collect()
}

fn next_state(state: &WorldState) -> WorldState {
    state
        .iter()
        .map(|(point, ground_state)| {
            (
                point.clone(),
                match *ground_state {
                    GroundState::OpenGround => {
                        let tree_count = point
                            .adjacent()
                            .iter()
                            .filter_map(|point| state.get(point))
                            .filter(|other_ground_state| **other_ground_state == GroundState::Trees)
                            .count();
                        if tree_count >= 3 {
                            GroundState::Trees
                        } else {
                            GroundState::OpenGround
                        }
                    }
                    GroundState::Trees => {
                        let lumberyard_count = point
                            .adjacent()
                            .iter()
                            .filter_map(|point| state.get(point))
                            .filter(|other_ground_state| {
                                **other_ground_state == GroundState::LumberYard
                            })
                            .count();
                        if lumberyard_count >= 3 {
                            GroundState::LumberYard
                        } else {
                            GroundState::Trees
                        }
                    }
                    GroundState::LumberYard => {
                        let lumberyard_count = point
                            .adjacent()
                            .iter()
                            .filter_map(|point| state.get(point))
                            .filter(|other_ground_state| {
                                **other_ground_state == GroundState::LumberYard
                            })
                            .count();
                        let tree_count = point
                            .adjacent()
                            .iter()
                            .filter_map(|point| state.get(point))
                            .filter(|other_ground_state| **other_ground_state == GroundState::Trees)
                            .count();
                        if lumberyard_count >= 1 && tree_count >= 1 {
                            GroundState::LumberYard
                        } else {
                            GroundState::OpenGround
                        }
                    }
                },
            )
        })
        .collect()
}
