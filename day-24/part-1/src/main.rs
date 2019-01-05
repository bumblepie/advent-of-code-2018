#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::collections::{ HashMap, HashSet };
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

use regex::Regex;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename).expect("Could not find file");
    let (lines, errors): (Vec<_>, Vec<_>) = BufReader::new(file).lines().partition(Result::is_ok);
    if !errors.is_empty() {
        for err in errors.into_iter().filter_map(Result::err) {
            eprintln!("{}", err);
        }
        process::exit(1);
    }
    let lines = lines.into_iter().filter_map(Result::ok).collect();
    let mut army_groups = read_armies_from_input(lines);

//    let mut s= String::new();

    loop {
        println!("\n\nNEXT TICK");
        for group in army_groups.iter() {
            println!("{:?}", group);
        }

        // select targets
        let mut targets: HashMap<usize, usize> = HashMap::new();
        let mut sorted_army_groups: Vec<usize> = (0..army_groups.len()).collect();
        sorted_army_groups.sort_by_key(|&index| {
            let army_group = army_groups.get(index).unwrap();
            (-army_group.effective_power(), -army_group.initiative)
        });

        for &index in sorted_army_groups.iter() {
            let army_group = army_groups.get(index).unwrap();
            if army_group.num_units > 0 {
                let possible_targets: Vec<(usize, &ArmyGroup)> = army_groups.iter()
                    .enumerate()
                    .filter(|(_other_index, other)| other.team != army_group.team)
                    .filter(|(_other_index, other)| other.num_units > 0)
                    .filter(|(other_index, _other)| !targets.values().collect::<Vec<_>>().contains(&other_index))
                    .collect();
                if let Some(target) = army_group.select_target(&possible_targets) {
                    println!("{} targets {}", index, target);
                    targets.insert(index, target);
                }
            }
        }

        // attack targets
        sorted_army_groups.sort_by_key(|&index| {
            let army_group = army_groups.get(index).unwrap();
            -army_group.initiative
        });

        for &index in sorted_army_groups.iter() {
            let army_group = army_groups.get(index).unwrap();
            if army_group.num_units > 0 {
                if let Some(&target_index) = targets.get(&index) {
                    let target = army_groups.get(target_index).unwrap();
                    let damage = army_group.damage_to(target);
                    let units_lost = i32::min(target.num_units, damage / target.hit_points);
                    let target = army_groups.get_mut(target_index).unwrap();
                    target.num_units -= units_lost;
                    println!("{} deals {} damage to {} killing {} units", index, damage, target_index, units_lost);
                }
            }
        }

        let teams_left: HashSet<&String> = army_groups.iter()
            .filter(|army_group| army_group.num_units > 0)
            .map(|army_group| &army_group.team)
            .collect();
//        std::io::stdin().read_line(&mut s);

        if teams_left.len() <= 1 {
            break;
        }
    }

    let units_count: i32 = army_groups.iter()
        .filter(|army_group| army_group.num_units > 0)
        .map(|army_group| army_group.num_units)
        .sum();
    println!("{}", units_count);
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct ArmyGroup {
    num_units: i32,
    hit_points: i32,
    team: String,
    attack_damage: i32,
    attack_type: String,
    initiative: i32,
    weaknesses: Vec<String>,
    immunities: Vec<String>,
}

impl ArmyGroup {

    fn select_target(&self, possible_targets: &Vec<(usize, &ArmyGroup)>) -> Option<usize> {
        possible_targets
            .into_iter()
            .filter(|(_index, army_group)|
                !army_group
                    .immunities
                    .contains(&self.attack_type)
            )
            .max_by_key(|(index, army_group)|
                (
                    self.damage_to(army_group),
                    army_group.effective_power(),
                    army_group.initiative,
                )
            )
            .map(|(index, _army_group)| *index)
    }

    fn damage_to(&self, other: &ArmyGroup) -> i32 {
        let damage_multiplier;
        if other.weaknesses.contains(&self.attack_type) {
            damage_multiplier = 2;
        } else if other.immunities.contains(&self.attack_type) {
            damage_multiplier = 0;
        } else {
            damage_multiplier = 1;
        }
        damage_multiplier * self.num_units * self.attack_damage
    }

    fn effective_power(&self) -> i32 {
        self.num_units * self.attack_damage
    }
}

fn read_armies_from_input(lines: Vec<String>) -> Vec<ArmyGroup> {
    lazy_static! {
        static ref army_name_regex: Regex = Regex::new(r"^(?P<army_name>[\w\s]+):$").unwrap();
    }
    let mut current_army = None;
    lines
        .into_iter()
        .filter_map(|line| {
            if line.is_empty() {
                None
            } else if let Some(captures) = army_name_regex.captures(&line) {
                current_army = captures
                    .name("army_name")
                    .map(|army_name| army_name.as_str().to_string());
                None
            } else {
                Some(read_army_group_from_line(&line, current_army.clone().unwrap()))
            }
        }).collect()
}

fn read_army_group_from_line(line: &String, current_army: String) -> ArmyGroup {
    lazy_static! {
        static ref army_group_regex: Regex = Regex::new(r"^(?P<num_units>\d+) units each with (?P<hit_points>\d+) hit points (\((?P<weaknesses_and_immunities>.*)\) )?with an attack that does (?P<attack_damage>\d+) (?P<attack_type>\w+) damage at initiative (?P<initiative>\d+)$").unwrap();
        static ref weaknesses_and_immunities_regex: Regex = Regex::new(r"^(((weak to (?P<weaknesses>(\w+(, )?)+))|(immune to (?P<immunities>(\w+(, )?)+)))(; )?){1,2}$").unwrap();
    }

    let captures = army_group_regex.captures(line).unwrap();
    let mut army_group = ArmyGroup {
        num_units: captures["num_units"].parse::<i32>().unwrap(),
        team: current_army,
        hit_points: captures["hit_points"].parse::<i32>().unwrap(),
        attack_damage: captures["attack_damage"].parse::<i32>().unwrap(),
        attack_type: captures["attack_type"].to_string(),
        initiative: captures["initiative"].parse::<i32>().unwrap(),
        weaknesses: Vec::new(),
        immunities: Vec::new(),
    };
    if let Some(weaknesses_and_immunities_string) = captures.name("weaknesses_and_immunities") {
        if let Some(captures) =
            weaknesses_and_immunities_regex.captures(weaknesses_and_immunities_string.as_str())
        {
            if let Some(weaknesses) = captures.name("weaknesses") {
                for weakness in weaknesses.as_str().split(", ") {
                    army_group.weaknesses.push(weakness.to_string());
                }
            }
            if let Some(immunities) = captures.name("immunities") {
                for immunity in immunities.as_str().split(", ") {
                    army_group.immunities.push(immunity.to_string());
                }
            }
        }
    }
    army_group
}
