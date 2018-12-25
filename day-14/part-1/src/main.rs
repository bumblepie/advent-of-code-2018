use std::env;

fn main() -> Result<(),Box<std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let target_recipe_number = args[1].parse::<usize>()?;

    let mut recipe_scores: Vec<usize> = vec![3,7];
    let mut elf_recipe_indexes: Vec<usize> = vec![0,1];

    while recipe_scores.len() < (target_recipe_number + 10) {
        let mut selected_recipes_sum: usize = elf_recipe_indexes.iter().map(|index| recipe_scores.get(*index).unwrap()).sum();
        let mut new_recipes = Vec::new();
        while selected_recipes_sum >= 10 {
            new_recipes.push(selected_recipes_sum % 10);
            selected_recipes_sum /= 10;
        }
        new_recipes.push(selected_recipes_sum % 10);
        new_recipes.reverse();
        recipe_scores.append(&mut new_recipes);

        elf_recipe_indexes = elf_recipe_indexes.into_iter().map(|index| {
            (index + recipe_scores.get(index).unwrap() + 1) % recipe_scores.len()
        }).collect();
    }
    println!("{}", recipe_scores[target_recipe_number..target_recipe_number+10].to_vec().iter().map(usize::to_string).collect::<Vec<String>>().join(""));
    Ok(())
}
