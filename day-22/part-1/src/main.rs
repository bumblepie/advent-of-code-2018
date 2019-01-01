use std::collections::HashMap;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let depth = args[1].parse::<i32>().expect("Unable to parse depth");
    let target_x = args[2].parse::<i32>().expect("Unable to parse target's x coordinate");
    let target_y = args[3].parse::<i32>().expect("Unable to parse target's y coordinate");
    let target_coordinates = (target_x, target_y);

    println!("{:?}", target_coordinates);

    let mut erosion_levels = HashMap::new();
    let mut risk_level = 0;

    for y in 0..target_coordinates.1 + 1 {
        for x in 0..target_coordinates.0 + 1 {
            // Note: (a * b) mod n == ((a mod n) * (b mod n)) mod n
            let index = match (x, y) {
                (0,0) => 0,
                coordinates if coordinates == target_coordinates => 0,
                (x, 0) => (x * 16807),
                (0, y) => (y * 48271),
                (x, y) => {
                    (erosion_levels.get(&(x-1, y)).unwrap() * erosion_levels.get(&(x, y-1)).unwrap())
                }
            };
            let erosion_level = (index + depth) % 20183;
            erosion_levels.insert((x,y), erosion_level);
            let region_type = match erosion_level % 3 {
                0 => RegionType::Rocky,
                1 => RegionType::Wet,
                2 => RegionType::Narrow,
                _ => panic!(format!("Unknown region type at ({},{})", x, y))
            };
            risk_level += match region_type {
                RegionType::Rocky => 0,
                RegionType::Wet => 1,
                RegionType::Narrow => 2,
            };
            print!("{}", match region_type {
                RegionType::Rocky => ".",
                RegionType::Wet => "=",
                RegionType::Narrow => "|",
            });
        }
        print!("\n");
    }
    println!("{}", risk_level);
}

enum RegionType {
    Rocky,
    Wet,
    Narrow
}
