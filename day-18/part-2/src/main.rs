use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let target_minutes = args[2]
        .parse::<i32>()
        .expect("Unable to parse target minutes");
    let file = File::open(filename).expect("Unable to open file");
    let (lines, errs): (Vec<_>, Vec<_>) = BufReader::new(file).lines().partition(Result::is_ok);
    if !errs.is_empty() {
        for err in errs.into_iter().filter_map(Result::err) {
            eprintln!("{}", err);
        }
        process::exit(1);
    }

    let initial_state = WorldState {
        world_map: lines
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
            .collect(),
    };

    let mut state = initial_state.clone();

    let mut duplicate_state_found = false;
    let mut states = HashMap::new();
    let mut minutes_count = 0;
    let mut hash = 0;
    while !duplicate_state_found {
        minutes_count += 1;
        state = next_state(&state);
        let lumberyard_count = state
            .world_map
            .iter()
            .filter(|(_point, ground_state)| **ground_state == GroundState::LumberYard)
            .count();

        let trees_count = state
            .world_map
            .iter()
            .filter(|(_point, ground_state)| **ground_state == GroundState::Trees)
            .count();
        println!(
            "At {}s T: {}, L: {}",
            minutes_count, trees_count, lumberyard_count
        );

        let mut hasher = DefaultHasher::new();
        state.hash(&mut hasher);
        hash = hasher.finish();

        duplicate_state_found = states.keys().collect::<HashSet<_>>().contains(&hash);
        if !(duplicate_state_found) {
            states.insert(hash, (minutes_count, trees_count, lumberyard_count));
        }
    }
    println!("Duplicate state found at {}", minutes_count);
    let duplicate_minutes_count = minutes_count;
    let original_minutes_count = states.get(&hash).unwrap().0;
    let diff = duplicate_minutes_count - original_minutes_count;
    let same_minute = ((target_minutes - original_minutes_count) % diff) + original_minutes_count;
    let reversed_states: HashMap<i32, (usize, usize)> = states
        .into_iter()
        .map(|(_hash, (seconds_count, trees_count, lumberyard_count))| {
            (seconds_count, (lumberyard_count, trees_count))
        })
        .collect();
    let (lumberyard_count, trees_count) = reversed_states.get(&same_minute).unwrap();
    println!(
        "{} Lumberyards * {} Trees = {}",
        lumberyard_count,
        trees_count,
        lumberyard_count * trees_count
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorldState {
    world_map: HashMap<Point, GroundState>,
}

impl Hash for WorldState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut points = self.world_map.iter().collect::<Vec<_>>();
        points.sort_by_key(|(point, _gs)| *point);
        points.iter().for_each(|(point, gs)| {
            point.hash(state);
            gs.hash(state);
        });
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
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

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
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

fn format_world_state(state: &WorldState) -> String {
    let mut result = String::new();
    let (x_min, x_max, y_min, y_max) = state.world_map.iter().fold(
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
        result += &format!(
            "{}\n",
            (x_min..x_max + 1)
                .map(|x| state
                    .world_map
                    .get(&Point { x, y })
                    .unwrap_or(&GroundState::OpenGround))
                .map(|ground_state| ground_state.to_string())
                .collect::<Vec<String>>()
                .join("")
        );
    }
    result
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
    WorldState {
        world_map: state
            .world_map
            .iter()
            .map(|(point, ground_state)| {
                (
                    point.clone(),
                    match *ground_state {
                        GroundState::OpenGround => {
                            let tree_count = point
                                .adjacent()
                                .iter()
                                .filter_map(|point| state.world_map.get(point))
                                .filter(|other_ground_state| {
                                    **other_ground_state == GroundState::Trees
                                })
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
                                .filter_map(|point| state.world_map.get(point))
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
                                .filter_map(|point| state.world_map.get(point))
                                .filter(|other_ground_state| {
                                    **other_ground_state == GroundState::LumberYard
                                })
                                .count();
                            let tree_count = point
                                .adjacent()
                                .iter()
                                .filter_map(|point| state.world_map.get(point))
                                .filter(|other_ground_state| {
                                    **other_ground_state == GroundState::Trees
                                })
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
            .collect(),
    }
}
