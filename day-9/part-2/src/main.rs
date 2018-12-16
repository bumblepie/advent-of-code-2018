use std::collections::{HashMap, VecDeque};
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<Error>>{
    let args: Vec<String> = env::args().collect();
    let game_params = GameParameters{
        num_players: args[1].parse::<u32>()?,
        final_marble: args[2].parse::<u32>()?,
    };

    let mut marble_circle = VecDeque::new();
    marble_circle.push_front(0);
    let mut scores: HashMap<u32, u32> = HashMap::new();
    for marble in 1..game_params.final_marble + 1 {
        if marble % 23 == 0 {
            //Special scoring
            rotate(&mut marble_circle, 7, Direction::CounterClockwise);
            let score_increase = marble + marble_circle.pop_front().unwrap_or(0);
            *scores.entry(marble % game_params.num_players).or_insert(0) += score_increase;
        } else {
            rotate(&mut marble_circle, 2, Direction::Clockwise);
            marble_circle.push_front(marble);
        }
    }
    println!("{:?}", marble_circle);
    println!("{:?}", scores);
    println!("{:?}", scores.iter().max_by_key(|(_player, &score)| score));
    Ok(())
}

struct GameParameters {
    num_players: u32,
    final_marble: u32,
}

enum Direction {
    Clockwise,
    CounterClockwise,
}

fn rotate<T>(circle: &mut VecDeque<T>, rotations: u32, direction: Direction) {
    for _ in 0..rotations {
        let element = match &direction {
            Direction::Clockwise => circle.pop_front(),
            Direction::CounterClockwise => circle.pop_back(),
        };
        match (element, &direction) {
            (Some(element), Direction::Clockwise) => circle.push_back(element),
            (Some(element), Direction::CounterClockwise) => circle.push_front(element),
            _ => ()
        }
    }
}