use std::cmp::Ordering;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename)?;
    let (lines, errors): (Vec<_>, Vec<_>) = BufReader::new(file).lines().partition(Result::is_ok);
    if !errors.is_empty() {
        for err in errors.into_iter().filter_map(Result::err) {
            eprintln!("{}", err);
        }
        process::exit(1);
    }
    let initial_world_state = lines
        .into_iter()
        .filter_map(Result::ok)
        .enumerate()
        .map(read_world_state_from_line)
        .fold(
            WorldState {
                walls: HashSet::new(),
                units: Vec::new(),
                bounds: Point { x: 0, y: 0 },
                elf_attack: 3,
                goblin_attack: 3,
            },
            |mut current_state, more_state| {
                current_state.append(more_state);
                current_state
            },
        );
    let mut lower_bound_attack = 0;
    let mut upper_bound_attack = 200;

    while (upper_bound_attack - lower_bound_attack) > 1 {
        let attack = (upper_bound_attack - lower_bound_attack) / 2 + lower_bound_attack;
        let mut world_state = initial_world_state.clone();
        world_state.elf_attack = attack;
        let mut combat_over = false;
        let mut dead_elf = false;
        while !combat_over  {
            combat_over = !world_state.tick();
            dead_elf = world_state.units.iter()
                .filter(|unit| unit.team == UnitTeam::Elf)
                .filter(|unit| unit.health <= 0)
                .next()
                .is_some();
        }
        if dead_elf {
            println!("Attack value {}: elf died", attack);
            lower_bound_attack = attack;
        } else {
            println!("Attack value {}: flawless", attack);
            upper_bound_attack = attack;
        }

    }
    let chosen_attack_value = lower_bound_attack + 1;
    println!("Lowest possible flawless: {}", chosen_attack_value);
    let mut world_state = initial_world_state.clone();
    world_state.elf_attack = chosen_attack_value;
    let mut ticks = 0;
    while world_state.tick() {
        ticks += 1;
    }

    let health_sum = world_state
        .units
        .iter()
        .map(|unit| unit.health)
        .filter(|&health| health > 0)
        .sum::<i32>();
    println!("{}", health_sum);
    println!("{}", ticks);
    let result: i32 = ticks * health_sum;
    println!("{}", result);
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn adjacent_points(&self) -> Vec<Point> {
        let mut result = vec![
            Point {
                x: self.x + 1,
                y: self.y,
            },
            Point {
                y: self.y + 1,
                x: self.x,
            },
        ];
        if self.x > 0 {
            result.push(Point {
                x: self.x - 1,
                y: self.y,
            });
        }
        if self.y > 0 {
            result.push(Point {
                y: self.y - 1,
                x: self.x,
            });
        }
        result
    }

    fn manhattan_distance(&self, other: &Point) -> i32 {
        i32::abs(self.x as i32 - other.x as i32) + i32::abs(self.y as i32 - other.y as i32)
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Point) -> Ordering {
        if self.y != other.y {
            self.y.cmp(&other.y)
        } else {
            self.x.cmp(&other.x)
        }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug)]
struct WorldState {
    walls: HashSet<Point>,
    units: Vec<Unit>,
    bounds: Point,
    elf_attack: i32,
    goblin_attack: i32,
}

impl WorldState {
    fn append(&mut self, mut other: WorldState) {
        self.walls.extend(&mut other.walls.into_iter());
        self.units.append(&mut other.units);
        self.bounds = Point {
            x: usize::max(self.bounds.x, other.bounds.x),
            y: usize::max(self.bounds.y, other.bounds.y),
        }
    }

    // Returns whether the combat is over
    fn tick(&mut self) -> bool {
        // Create a combined list of elf and goblin refs
        let mut sorted_units: Vec<(usize, &Unit)> = self.units.iter().enumerate().collect();

        // Sort list based on position
        sorted_units.sort_by_key(|(_unit_index, unit)| &unit.position);

        // Use indexes to avoid borrow checker shenanigans
        let sorted_units_indexes: Vec<usize> = sorted_units
            .into_iter()
            .map(|(unit_index, _unit)| unit_index)
            .collect();
        // Perform turn for each unit on list
        for unit_index in sorted_units_indexes {
            let unit = self.units.get(unit_index).unwrap();

            //Ignore if unit is dead
            if unit.health <= 0 {
                continue;
            }

            //            println!("Unit {:?} at {:?}'s turn begins", unit.team, unit.position);
            // Find targets
            let possible_targets: Vec<&Unit> = self
                .units
                .iter()
                .filter(|other_unit| unit.team != other_unit.team)
                .filter(|unit| unit.health > 0)
                .collect();
            //                        println!("{:?}", possible_targets);

            // Combat over if no possible targets found
            if possible_targets.is_empty() {
                //                                println!("Combat over");
                return false;
            }

            let possible_attack_squares: Vec<Point> = possible_targets
                .into_iter()
                .flat_map(|unit| unit.position.adjacent_points())
                .collect();
            //            println!("{:?}", possible_attack_squares);

            // Filter out reachable squares, find nearest ones
            let (mut nearest_attack_squares, min_distance_to_an_attack_square) =
                possible_attack_squares
                    .into_iter()
                    .filter_map(|square| {
                        match self.distance_between_points(&unit.position, &square, unit) {
                            Some(distance) => Some((square, distance)),
                            None => None,
                        }
                    })
                    .fold(
                        (Vec::new(), std::i32::MAX),
                        |(mut attack_squares, min_distance), (new_attack_square, new_distance)| {
                            if new_distance < min_distance {
                                attack_squares.clear();
                                attack_squares.push(new_attack_square);
                                (attack_squares, new_distance)
                            } else if new_distance == min_distance {
                                attack_squares.push(new_attack_square);
                                (attack_squares, min_distance)
                            } else {
                                (attack_squares, min_distance)
                            }
                        },
                    );
            //                        println!("{:?}", nearest_attack_squares);

            // Select attack square
            nearest_attack_squares.sort();
            let chosen_attack_square = nearest_attack_squares.first();
            //                        println!("{:?}", chosen_attack_square);

            // Select best adjacent square to move into
            if min_distance_to_an_attack_square > 0 {
                if let Some(attack_square) = chosen_attack_square {
                    let mut candidate_squares = vec![unit.position.clone()];
                    candidate_squares.append(&mut unit.position.adjacent_points());
                    let (mut best_moves, _min_distance) = candidate_squares
                        .into_iter()
                        .filter_map(|candidate| {
                            match self.distance_between_points(&candidate, attack_square, unit) {
                                Some(distance) => Some((candidate, distance)),
                                None => None,
                            }
                        })
                        .fold(
                            (Vec::new(), std::i32::MAX),
                            |(mut min_squares, min_distance), (candidate, distance)| {
                                if distance < min_distance {
                                    min_squares.clear();
                                    min_squares.push(candidate);
                                    (min_squares, distance)
                                } else if distance == min_distance {
                                    min_squares.push(candidate);
                                    (min_squares, min_distance)
                                } else {
                                    (min_squares, min_distance)
                                }
                            },
                        );
                    best_moves.sort();
                    if let Some(chosen_move) = best_moves.first() {
                        // Shadow with a mutable reference after immutable ref no longer needed
                        let mut unit = self.units.get_mut(unit_index).unwrap();
                        unit.position = chosen_move.clone();
                    }
                }
            }

            // Remake immutable ref for attack phase
            let unit = self.units.get(unit_index).unwrap();

            // Attack if we can
            if min_distance_to_an_attack_square <= 1 {
                let mut possible_targets: Vec<(usize, &Unit)> = self
                    .units
                    .iter()
                    .enumerate()
                    .filter(|(_index, other_unit)| {
                        other_unit.position.manhattan_distance(&unit.position) == 1
                    })
                    .filter(|(_index, other_unit)| other_unit.team != unit.team)
                    .filter(|(_index, unit)| unit.health > 0)
                    .collect();
                possible_targets.sort_by(|(_index, unit), (_index_other, other)| {
                    if unit.health == other.health {
                        unit.position.cmp(&other.position)
                    } else {
                        unit.health.cmp(&other.health)
                    }
                });
                if let Some((index, _attack_target)) = possible_targets.first() {
                    // Clone index and attack value as it is a reference to units atm - this allows us to mutably borrow at end of this block
                    let index = index.clone();
                    let attack_value = match unit.team {
                        UnitTeam::Elf => self.elf_attack,
                        UnitTeam::Goblin => self.goblin_attack,
                    };
                    let attack_target = self.units.get_mut(index).unwrap();
                    attack_target.health -= attack_value;
                }
            }
        }
        true
    }

    fn distance_between_points(&self, from: &Point, to: &Point, unit: &Unit) -> Option<i32> {
        let all_non_passable_squares: HashSet<Point> = self
            .walls
            .iter()
            .chain(
                self.units
                    .iter()
                    .filter(|unit| unit.health > 0)
                    .filter(|other_unit| other_unit.position != unit.position)
                    .map(|unit| &unit.position),
            )
            .map(|point| point.clone())
            .collect();

        // Ensure target and starting square are reachable
        if all_non_passable_squares.contains(to) || all_non_passable_squares.contains(from) {
            return None;
        }

        let mut stack: Vec<(Point, i32, i32)> =
            vec![(from.clone(), 0, from.manhattan_distance(to))];
        let mut checked_squares = HashSet::new();
        while !stack.is_empty() {
            // Sort by negative ~~manhattan~~ distance_travelled so lowest is at back to be popped
            stack.sort_by_key(|(_point, distance_from_origin, _manhattan)| -distance_from_origin);
            let (next_point, distance_from_origin, manhattan_distance) = stack.pop().unwrap();

            // Skip if already checked
            if checked_squares.contains(&next_point) {
                continue;
            }

            if manhattan_distance == 0 {
                return Some(distance_from_origin);
            } else {
                stack.append(
                    &mut next_point
                        .adjacent_points()
                        .into_iter()
                        .filter(|point| !all_non_passable_squares.contains(point))
                        .filter(|point| !checked_squares.contains(point))
                        .filter(|point| point.x <= self.bounds.x && point.y <= self.bounds.y)
                        .map(|point| {
                            let manhattan_distance = point.manhattan_distance(to);
                            (point, distance_from_origin + 1, manhattan_distance)
                        })
                        .collect(),
                );
                checked_squares.insert(next_point);
            }
        }
        None
    }
}

impl fmt::Display for WorldState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut display_string = (0..self.bounds.y + 1)
            .map(|y| {
                (0..self.bounds.x)
                    .map(|x| {
                        let point = Point { x, y };
                        if self.walls.contains(&point) {
                            String::from("#")
                        } else {
                            match self
                                .units
                                .iter()
                                .filter(|unit| unit.position == point)
                                .filter(|unit| unit.health > 0)
                                .next()
                            {
                                Some(unit) => match unit.team {
                                    UnitTeam::Elf => String::from("E"),
                                    UnitTeam::Goblin => String::from("G"),
                                },
                                None => String::from("."),
                            }
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("")
            })
            .collect::<Vec<String>>()
            .join("\n");
        display_string += &format!(
            "{:?}",
            self.units
                .iter()
                .map(|unit| unit.health)
                .collect::<Vec<i32>>()
        );
        writeln!(f, "{}", display_string)
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum UnitTeam {
    Elf,
    Goblin,
}
#[derive(Clone, Debug)]
struct Unit {
    position: Point,
    health: i32,
    team: UnitTeam,
}

fn read_world_state_from_line((y, line): (usize, String)) -> WorldState {
    const STARTING_HEALTH: i32 = 200;

    let mut walls = HashSet::new();
    let mut units = Vec::new();
    for (x, c) in line.chars().enumerate() {
        let point = Point { x, y };
        match c {
            '#' => {
                walls.insert(point);
            }
            'E' => {
                units.push(Unit {
                    position: point,
                    health: STARTING_HEALTH,
                    team: UnitTeam::Elf,
                });
            }
            'G' => {
                units.push(Unit {
                    position: point,
                    health: STARTING_HEALTH,
                    team: UnitTeam::Goblin,
                });
            }
            _ => (),
        }
    }
    WorldState {
        walls,
        units,
        bounds: Point { x: line.len(), y },
        elf_attack: 0,
        goblin_attack: 0,
    }
}
