use std::collections::HashMap;
use std::env;

fn main() -> Result<(), Box<std::error::Error>> {
    const GRID_SIDE_SIZE: usize = 300;

    let args: Vec<String> = env::args().collect();
    let serial_number = args[1].parse::<i32>()?;
    let fuel_cell_power_grid =
        generate_fuel_cell_grid(GRID_SIDE_SIZE, GRID_SIDE_SIZE, serial_number);
    let sum_table = generate_sum_table_grid(&fuel_cell_power_grid);
    let result = get_highest_power_region(&sum_table);
    println!("{:?}", result);
    Ok(())
}

struct Grid {
    width: usize,
    height: usize,
    values: HashMap<(usize, usize), i32>,
}

fn get_fuel_cell_power(x: i32, y: i32, serial_number: i32) -> i32 {
    let rack_id = x + 10;
    let mut power_level = y * rack_id;
    power_level += serial_number;
    power_level *= rack_id;
    ((power_level / 100) % 10) - 5
}

fn generate_fuel_cell_grid(width: usize, height: usize, serial_number: i32) -> Grid {
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
        values: fuel_cells,
    }
}

fn generate_sum_table_grid(grid: &Grid) -> Grid {
    let mut values = HashMap::new();
    for y in 1..grid.height + 1 {
        let mut row_sum = 0;
        for x in 1..grid.width + 1 {
            row_sum += grid.values.get(&(x, y)).unwrap();
            let area_sum = values.get(&(x, y - 1)).unwrap_or(&0) + row_sum;
            values.insert((x, y), area_sum);
        }
    }
    Grid {
        width: grid.width,
        height: grid.height,
        values,
    }
}

fn get_highest_power_region(sum_table: &Grid) -> (usize, usize, usize, usize, i32) {
    (1..sum_table.width + 1)
        .map(|region_size| {
            get_highest_power_for_region_of_size(&sum_table, region_size, region_size)
        })
        .max_by_key(|(_x, _y, _width, _height, total)| *total)
        .unwrap()
}

fn get_highest_power_for_region_of_size(
    sum_table: &Grid,
    region_width: usize,
    region_height: usize,
) -> (usize, usize, usize, usize, i32) {
    let mut max_power = (0, 0, 0, 0, std::i32::MIN);

    for y in 1..sum_table.height - region_height {
        for x in 1..sum_table.width - region_width {
            let top_left_area_sum = sum_table.values.get(&(x - 1, y - 1)).unwrap_or(&0);
            let left_area_sum = sum_table.values.get(&(x - 1, y + region_height - 1)).unwrap_or(&0);
            let top_area_sum = sum_table.values.get(&(x + region_width - 1, y - 1)).unwrap_or(&0);
            let all_area_sum = sum_table
                .values
                .get(&(x + region_width - 1, y + region_height - 1))
                .unwrap_or(&0);
            let region_sum = top_left_area_sum + all_area_sum - left_area_sum - top_area_sum;
            if region_sum > max_power.4 {
                max_power = (x, y, region_width, region_height, region_sum);
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
    fn sum_table_generation() {
        let mut values = HashMap::new();
        values.insert((1, 1), 1);
        values.insert((2, 1), 2);
        values.insert((3, 1), 3);
        values.insert((1, 2), 4);
        values.insert((2, 2), 5);
        values.insert((3, 2), 6);
        values.insert((1, 3), 7);
        values.insert((2, 3), 8);
        values.insert((3, 3), 9);
        let grid = Grid {
            width: 3,
            height: 3,
            values,
        };
        let sum_table = generate_sum_table_grid(&grid);
        assert_eq!(sum_table.width, 3);
        assert_eq!(sum_table.height, 3);
        assert_eq!(sum_table.values.get(&(1, 1)).unwrap(), &1);
        assert_eq!(sum_table.values.get(&(3, 1)).unwrap(), &6);
        assert_eq!(sum_table.values.get(&(2, 2)).unwrap(), &12);
        assert_eq!(sum_table.values.get(&(1, 3)).unwrap(), &12);
        assert_eq!(sum_table.values.get(&(3, 3)).unwrap(), &45);
    }

    #[test]
    fn max_power_calculations() {
        let grid = generate_fuel_cell_grid(300, 300, 18);
        assert_eq!(
            get_highest_power_region(&generate_sum_table_grid(&grid)),
            (90, 269, 16, 16, 113)
        );
        let grid = generate_fuel_cell_grid(300, 300, 42);
        assert_eq!(
            get_highest_power_region(&generate_sum_table_grid(&grid)),
            (232, 251, 12, 12, 119)
        );
    }
}
