#[macro_use]
extern crate criterion;

use criterion::Criterion;

#[derive(Clone)]
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

fn criterion_benchmark(c: &mut Criterion) {
    const SIZE: usize = 1000;
    const START: usize = 1000000;
    let mut initial_state = State {
        recipe_scores: vec![3, 7],
        elf_recipe_indexes: vec![0, 1],
    };
    let target_recipe_sequence = [11,11,11,11,11,11];
    let mut _found_sequence = false;
    for _ in 0..START {
        initial_state = next_state(initial_state);
    }
    c.bench_function(&format!("Next {} states after {}", SIZE, START), move |b| b.iter(|| {
        let mut state = initial_state.clone();
        for _ in 0..SIZE {
            state = next_state(state);
            if state.recipe_scores.len() >= target_recipe_sequence.len() && state.recipe_scores[state.recipe_scores.len() - 1] == target_recipe_sequence[target_recipe_sequence.len() - 1] {
                _found_sequence = state.recipe_scores[state.recipe_scores.len() - target_recipe_sequence.len()..].to_vec() == target_recipe_sequence
            }
        }
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);