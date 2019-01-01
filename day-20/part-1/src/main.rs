extern crate unicode_reader;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use unicode_reader::Graphemes;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename).expect("Unable to open file");

    // Assumptions in order for this to work:
    //  - All possible paths are given by the input
    //  - The paths don't loop back on themselves except with explicit branches
    let rooms = get_paths_from_file(file);
    println!("Furthest room: {:?}", rooms.iter().max_by_key(|room| room.distance_from_start));
    println!("Number of rooms at least 1000 doors away: {}", rooms.iter().filter(|room| room.distance_from_start >= 1000).count());
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Path {
    steps: Vec<Direction>,
    current_position: Point,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Room {
    position: Point,
    distance_from_start: usize,
}


impl Path {

    // Returns the room at the current point, with the distance based on the path's length
    fn step(&mut self, step: Direction) -> Room {
        let room = self.get_room();

        match step {
            Direction::North => self.current_position.y -= 1,
            Direction::East => self.current_position.x += 1,
            Direction::South => self.current_position.y += 1,
            Direction::West => self.current_position.x -= 1,
        }

        match self.steps.last() {
            Some(direction) => {
                if direction.cancels_out(&step) {
                    self.steps.pop();
                } else {
                    self.steps.push(step);
                }
            }
            None => {
                self.steps.push(step);
            },
        }
        room
    }

    // Create a Room from the path
    fn get_room(&self) -> Room {
        Room {
            position: self.current_position.clone(),
            distance_from_start: self.steps.len(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn cancels_out(&self, other: &Direction) -> bool {
        match self {
            Direction::North => *other == Direction::South,
            Direction::East => *other == Direction::West,
            Direction::South => *other == Direction::North,
            Direction::West => *other == Direction::East,
        }
    }
}

#[derive(Debug)]
struct Branch {
    starting_points: Vec<Path>,
    end_points: Vec<Path>,
}

fn get_paths_from_file(file: File) -> Vec<Room> {
    // init

    // The path heads that we are currently exploring
    let mut path_heads: Vec<Path> = vec![Path {
        steps: Vec::new(),
        current_position: Point { x: 0, y: 0 },
    }];

    // Saves all path heads upon starting a group of branching paths in a stack (for nested groups).
    // When we start trying a new branch, we need to clone these as a starting point
    // When the current group is completed, we pop the top of the stack
    let mut branches = Vec::new();

    // Rooms which have been visited by a path that then backtracks out of it and continues on
    // Eg: a path that goes WWNSWW has an offshoot room between the west moves
    let mut all_rooms = HashMap::new();

    for next_char in Graphemes::from(file).filter_map(Result::ok) {
        match next_char.as_ref() {
            "N" => path_heads
                .iter_mut()
                .for_each(|path| add_room(&mut all_rooms, path.step(Direction::North))),
            "E" => path_heads
                .iter_mut()
                .for_each(|path| add_room(&mut all_rooms, path.step(Direction::East))),
            "S" => path_heads
                .iter_mut()
                .for_each(|path| add_room(&mut all_rooms, path.step(Direction::South))),
            "W" => path_heads
                .iter_mut()
                .for_each(|path| add_room(&mut all_rooms, path.step(Direction::West))),
            "(" => {
                // Initialise branch
                branches.push(Branch {
                    starting_points: path_heads.clone(),
                    end_points: Vec::new(),
                });
            },
            "|" => {
                // Add all end points for this branch
                branches.last_mut().unwrap().end_points.append(&mut path_heads.clone());
                // Reset path heads to search from start of branch
                path_heads = branches.last().unwrap().starting_points.clone();
            }
            ")" => {
                // Add final end points for this branch
                branches.last_mut().unwrap().end_points.append(&mut path_heads.clone());

                // Keep only shortest versions of the paths to the branch's end points
                // Also remove branch from stack as we are done with it
                path_heads = branches.pop().unwrap().end_points.into_iter().fold(HashMap::<Point, Path>::new(), |mut map, next_end_point| {
                    let is_shortest_path = match map.get(&next_end_point.current_position) {
                        Some(path) => if next_end_point.steps.len() < path.steps.len() {
                            true
                        } else {
                            false
                        }
                        None => true
                    };
                    if is_shortest_path {
                        map.insert(next_end_point.current_position.clone(), next_end_point);
                    }
                    map
                }).into_iter().map(|(_point, path)| path).collect();
            }
            "^" | "$" => (),
            _ => panic!("unrecognised character"),
        }
    }
    path_heads.iter().for_each(|path| add_room(&mut all_rooms, path.get_room()));
    all_rooms.into_iter().map(|(_point, room)| room).collect()
}

fn add_room(all_rooms: &mut HashMap<Point, Room>, new_room: Room) {
    match all_rooms.get(&new_room.position) {
        Some(existing_room) => {
            if new_room.distance_from_start < existing_room.distance_from_start {
                all_rooms.insert(new_room.position.clone(), new_room);
            }
        },
        None => {
            all_rooms.insert(new_room.position.clone(), new_room);
        },
    }
}
