use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let depth = args[1].parse::<i32>().expect("Unable to parse depth");
    let target_x = args[2].parse::<i32>().expect("Unable to parse target's x coordinate");
    let target_y = args[3].parse::<i32>().expect("Unable to parse target's y coordinate");
    let target_coordinates = (target_x, target_y);

    println!("{:?}", target_coordinates);
    let result = part_2(target_coordinates, depth).unwrap();
    println!("{:?}", result);
//    println!("{}", result.description);

}

#[derive(Clone, Debug, Eq, PartialEq)]
struct State {
    position: (i32, i32),
    equipment: Equipment,
    minutes_spent: i32,
    region_type: RegionType,
//    description: String,
}

impl State {
    // Simplified version for checking previously visited states
    fn equivalent(&self) -> ((i32, i32), Equipment) {
        (self.position, self.equipment.clone())
    }
}

impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        // Reverse so that minimum becomes maximum for binary heap
        self.minutes_spent.cmp(&other.minutes_spent).reverse()
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RegionType {
    Rocky,
    Wet,
    Narrow
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum Equipment {
    Nothing,
    ClimbingGear,
    Torch,
}

fn can_traverse(region: &RegionType) -> HashSet<Equipment> {
    match region {
        RegionType::Rocky => vec![Equipment::ClimbingGear, Equipment::Torch].into_iter().collect(),
        RegionType::Wet => vec![Equipment::ClimbingGear, Equipment::Nothing].into_iter().collect(),
        RegionType::Narrow => vec![Equipment::Torch, Equipment::Nothing].into_iter().collect(),
    }
}

fn get_erosion_level((x,y): (i32, i32), target_coordinates: (i32, i32), depth: i32, mut erosion_levels: &mut HashMap<(i32, i32), i32>) -> i32 {
    let existing_erosion_level = erosion_levels.get(&(x, y));
    if existing_erosion_level.is_some() {
        existing_erosion_level.unwrap().clone()
    } else {
        let index = match (x, y) {
            (0,0) => 0,
            coordinates if coordinates == target_coordinates => 0,
            (x, 0) => (x * 16807),
            (0, y) => (y * 48271),
            (x, y) => {
                (get_erosion_level((x-1, y), target_coordinates, depth, &mut erosion_levels) * get_erosion_level((x, y-1), target_coordinates, depth, &mut erosion_levels))
            }
        };
        let erosion_level = (index + depth) % 20183;
        erosion_levels.insert((x.clone(),y.clone()), erosion_level);
        erosion_level
    }
}

fn get_region_type(erosion_level: i32) -> RegionType {
    match erosion_level % 3 {
        0 => RegionType::Rocky,
        1 => RegionType::Wet,
        2 => RegionType::Narrow,
        _ => panic!("Unknown region type"),
    }
}

fn part_2(target_coordinates: (i32, i32), depth: i32) -> Option<State> {
    let mut erosion_levels = HashMap::<(i32, i32), i32>::new();
    let mut visited_states = HashSet::new();
    let mut possible_states = BinaryHeap::<State>::new();
    possible_states.push(State {
        position: (0,0),
        region_type: RegionType::Rocky,
        minutes_spent: 0,
        equipment: Equipment::Torch,
//        description: String::from(""),
    });

    loop {
        let current_state = possible_states.pop().unwrap();
        if visited_states.contains(&current_state.equivalent()) {
            continue;
        }
        println!("Exploring state {:?}", current_state);
        visited_states.insert(current_state.equivalent());
        let position = current_state.position;

        if position == target_coordinates {
            if current_state.equipment != Equipment::Torch {
                possible_states.push(State {
                    position,
                    region_type: RegionType::Rocky,
                    minutes_spent: current_state.minutes_spent + 7,
                    equipment: Equipment::Torch,
//                    description: current_state.description.clone() + "\n Switch to torch",
                })
            } else {
                return Some(current_state)
            }
        }

        let new_positions = vec![
            (position.0-1, position.1),
            (position.0+1, position.1),
            (position.0, position.1-1),
            (position.0, position.1+1),
        ];
        new_positions.into_iter()
            .filter(|(x, y)| *x >= 0 && *y >= 0)
            .for_each(|position| {
                let erosion_level = get_erosion_level(position, target_coordinates, depth, &mut erosion_levels);
                let region_type = get_region_type(erosion_level);
                if can_traverse(&region_type).contains(&current_state.equipment) {
                    let next_state = State {
                        minutes_spent: current_state.minutes_spent + 1,
                        equipment: current_state.equipment.clone(),
//                        description: current_state.description.clone() + &format!("\n Move to {:?} ({:?})", position, region_type),
                        position,
                        region_type,
                    };
                    possible_states.push(next_state);
                }
            });
        for equipment in can_traverse(&current_state.region_type) {
            let next_state = State {
                position,
                region_type: current_state.region_type.clone(),
                minutes_spent: current_state.minutes_spent + 7,
//                description: current_state.description.clone() + &format!("\n Switch to {:?}", equipment),
                equipment,
            };
            possible_states.push(next_state);
        }

    }
    return None
}
