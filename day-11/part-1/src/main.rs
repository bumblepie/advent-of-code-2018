use std::collections::HashMap;
use std::env;

fn main() -> Result<(), Box<std::error::Error>> {
    const GRID_WIDTH: usize = 300;
    const GRID_HEIGHT: usize = 300;
    const REGION_WIDTH: usize = 3;
    const REGION_HEIGHT: usize = 3;

    let args: Vec<String> = env::args().collect();
    let serial_number = args[1].parse::<i32>()?;
    let grid = generate_grid(GRID_WIDTH, GRID_HEIGHT, serial_number);
    let result = get_highest_power_region(grid, REGION_WIDTH, REGION_HEIGHT);
    println!("{:?}", result);
    Ok(())
}

struct Grid {
    width: usize,
    height: usize,
    fuel_cells: HashMap<(usize, usize), i32>,
}

fn get_fuel_cell_power(x: i32, y: i32, serial_number: i32) -> i32 {
    let rack_id = x + 10;
    let mut power_level = y * rack_id;
    power_level += serial_number;
    power_level *= rack_id;
    ((power_level / 100) % 10) - 5
}

fn generate_grid(width: usize, height: usize, serial_number: i32) -> Grid {
    let mut fuel_cells = HashMap::new();
    for x in 1..width + 1 {
        for y in 1..height + 1 {
            fuel_cells.insert(
                (x, y),
                get_fuel_cell_power(x as i32, y as i32, serial_number),
            );
        }
    }
    Grid {
        width,
        height,
        fuel_cells,
    }
}

fn get_highest_power_region(
    grid: Grid,
    region_width: usize,
    region_height: usize,
) -> (usize, usize, i32) {
    let mut max_power = (0, 0, std::i32::MIN);

    for y in 1..grid.height - region_height {
        for x in 1..grid.width - region_width {
            let mut region_sum: i32 = 0;
            for local_x in x..x + region_width {
                for local_y in y..y + region_height {
                    region_sum += grid.fuel_cells.get(&(local_x, local_y)).unwrap();
                }
            }
            if region_sum > max_power.2 {
                max_power = (x, y, region_sum);
            }
        }
    }
    max_power
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_cell_calculations() {
        assert_eq!(get_fuel_cell_power(3, 5, 8), 4);
        assert_eq!(get_fuel_cell_power(122, 79, 57), -5);
        assert_eq!(get_fuel_cell_power(217, 196, 39), 0);
        assert_eq!(get_fuel_cell_power(101, 153, 71), 4);
    }

    #[test]
    fn max_power_calculations() {
        let grid = generate_grid(300, 300, 18);
        assert_eq!(get_highest_power_region(grid, 3, 3), (33, 45, 29));
        let grid = generate_grid(300, 300, 42);
        assert_eq!(get_highest_power_region(grid, 3, 3), (21, 61, 30));
    }
}
