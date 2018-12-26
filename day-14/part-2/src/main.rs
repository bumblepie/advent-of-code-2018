use std::env;

fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let target_recipe_sequence: Vec<usize> = args[1].chars().map(|c| c.to_digit(10).unwrap() as usize).collect();

    let mut state = State {
        recipe_scores: vec![3, 7],
        elf_recipe_indexes: vec![0, 1],
    };
    let mut found_sequence = false;
    let mut last_checked_index = 0;
    while !found_sequence {
        state = next_state(state);

        // While until we have enough
        while last_checked_index + target_recipe_sequence.len() < state.recipe_scores.len() && !found_sequence {
            if !found_sequence && state.recipe_scores[last_checked_index..last_checked_index + target_recipe_sequence.len()].to_vec() == target_recipe_sequence {
                found_sequence = true;
            } else {
                last_checked_index += 1;
            }
        }

    }
    println!("{}", last_checked_index);
    Ok(())
}

struct State {
    recipe_scores: Vec<usize>,
    elf_recipe_indexes: Vec<usize>,
}

fn next_state(old_state: State) -> State {
    let mut selected_recipes_sum: usize = old_state.elf_recipe_indexes
        .iter()
        .map(|index| old_state.recipe_scores.get(*index).unwrap())
        .sum();
    let mut new_recipes = Vec::new();
    while selected_recipes_sum >= 10 {
        new_recipes.push(selected_recipes_sum % 10);
        selected_recipes_sum /= 10;
    }
    new_recipes.push(selected_recipes_sum % 10);
    new_recipes.reverse();
    let mut all_new_recipes = old_state.recipe_scores;
    all_new_recipes.append(&mut new_recipes);

    let new_elf_recipe_indexes = old_state.elf_recipe_indexes
        .into_iter()
        .map(|index| (index + all_new_recipes.get(index).unwrap() + 1) % all_new_recipes.len())
        .collect();

    State {
        recipe_scores: all_new_recipes,
        elf_recipe_indexes: new_elf_recipe_indexes,
    }
}