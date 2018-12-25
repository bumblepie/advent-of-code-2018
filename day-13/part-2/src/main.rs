use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn main() -> Result<(), Box<std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename)?;
    let (lines, errors): (Vec<_>, Vec<_>) = BufReader::new(file).lines().partition(Result::is_ok);
    if !errors.is_empty() {
        panic!("Errors oh no");
    }
    let initial_state = lines
        .into_iter()
        .filter_map(Result::ok)
        .enumerate()
        .map(get_tracks_and_carts_from_line)
        .fold(
            WorldState {
                track_map: HashMap::new(),
                carts: HashMap::new(),
            },
            |mut state, (more_tracks, more_carts)| {
                state.track_map.extend(more_tracks);
                for cart in more_carts {
                    state.carts.insert(state.carts.len(), cart);
                }
                state
            },
        );
    let mut state = initial_state;
    let mut tick = 0;
    while state.carts.len() > 1 {
        tick += 1;
        state = next_world_state(&state);
//        print_state(&state);
//        std::io::stdin().read(&mut [0u8]).unwrap();
    }
    println!("Final cart at tick {} is {:?}", tick, state.carts.iter().next().unwrap());
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Point {
    x: usize,
    y: usize,
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
enum Track {
    Vertical,
    Horizontal,
    Intersection,
    CornerForwardSlash,
    CornerBackSlash,
}

#[derive(Clone, Debug)]
enum CartDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug)]
enum NextTurn {
    Left,
    Straight,
    Right,
}

fn right_turn(relative_to: &CartDirection) -> CartDirection {
    match relative_to {
        CartDirection::Up => CartDirection::Right,
        CartDirection::Down => CartDirection::Left,
        CartDirection::Left => CartDirection::Up,
        CartDirection::Right => CartDirection::Down,
    }
}

fn left_turn(relative_to: &CartDirection) -> CartDirection {
    match relative_to {
        CartDirection::Up => CartDirection::Left,
        CartDirection::Down => CartDirection::Right,
        CartDirection::Left => CartDirection::Down,
        CartDirection::Right => CartDirection::Up,
    }
}

#[derive(Clone, Debug)]
struct Cart {
    position: Point,
    direction: CartDirection,
    next_turn: NextTurn,
}

#[derive(Clone, Debug)]
struct MapPosition {
    position: Point,
    track: Option<Track>,
    cart: Option<Cart>,
}

#[derive(Clone)]
struct WorldState {
    track_map: HashMap<Point, Track>,
    carts: HashMap<usize, Cart>,
}

fn next_world_state(state: &WorldState) -> WorldState {
    let mut old_carts: Vec<usize> = state.carts.clone().into_iter().map(|(id, _cart)| id).collect();
    old_carts.sort_by_key(|cart_id| state.carts.get(&cart_id).unwrap().position.clone());
    old_carts.reverse();
    let mut new_carts= HashMap::new();

    while !old_carts.is_empty() {
        let cart_id = old_carts.pop().unwrap();
        let mut cart = state.carts.get(&cart_id).unwrap().clone();
        //Move cart to next position
        let new_position = match cart.direction {
            CartDirection::Up => Point { y: cart.position.y-1, ..cart.position },
            CartDirection::Down => Point { y: cart.position.y+1, ..cart.position },
            CartDirection::Left => Point { x: cart.position.x-1, ..cart.position },
            CartDirection::Right => Point { x: cart.position.x+1, ..cart.position },
        };
        if old_carts.iter().filter(|cart_id| state.carts.get(&cart_id).unwrap().position == new_position).next().is_some() {
            old_carts = old_carts.into_iter().filter(|cart_id| state.carts.get(&cart_id).unwrap().position != new_position).collect();
        } else if new_carts.iter().filter(|(_cart_id, new_cart): &(&usize, &Cart)| new_cart.position == new_position).next().is_some() {
            new_carts = new_carts.into_iter().filter(|(_cart_id, new_cart): &(usize, Cart)| new_cart.position != new_position).collect();
        } else {
            cart.position = new_position;


            // Turn cart if needed
            cart.direction = match state.track_map.get(&cart.position).unwrap() {
                Track::Vertical => cart.direction.clone(),
                Track::Horizontal => cart.direction.clone(),
                Track::CornerBackSlash => match cart.direction {
                    CartDirection::Up => CartDirection::Left,
                    CartDirection::Down => CartDirection::Right,
                    CartDirection::Left => CartDirection::Up,
                    CartDirection::Right => CartDirection::Down,
                },
                Track::CornerForwardSlash => match cart.direction {
                    CartDirection::Up => CartDirection::Right,
                    CartDirection::Down => CartDirection::Left,
                    CartDirection::Left => CartDirection::Down,
                    CartDirection::Right => CartDirection::Up,
                },
                Track::Intersection => {
                    match cart.next_turn {
                        NextTurn::Left => {
                            cart.next_turn = NextTurn::Straight;
                            left_turn(&cart.direction)
                        },
                        NextTurn::Straight => {
                            cart.next_turn = NextTurn::Right;
                            cart.direction.clone()
                        },
                        NextTurn::Right => {
                            cart.next_turn = NextTurn::Left;
                            right_turn(&cart.direction)
                        },
                    }
                }
            };
            new_carts.insert(cart_id, cart);
        }
    }
    WorldState {
        track_map: state.track_map.clone(),
        carts: new_carts,
    }
}

fn get_tracks_and_carts_from_line((line_y, line): (usize, String)) -> (HashMap<Point, Track>, Vec<Cart>) {
    line.chars()
        .enumerate()
        .filter_map(|(x, c)| {
            let position = Point { x, y: line_y };
            match c {
                '|' => Some(MapPosition {
                    position,
                    track: Some(Track::Vertical),
                    cart: None,
                }),
                '-' => Some(MapPosition {
                    position,
                    track: Some(Track::Horizontal),
                    cart: None,
                }),
                '+' => Some(MapPosition {
                    position,
                    track: Some(Track::Intersection),
                    cart: None,
                }),
                '/' => Some(MapPosition {
                    position,
                    track: Some(Track::CornerForwardSlash),
                    cart: None,
                }),
                '\\' => Some(MapPosition {
                    position,
                    track: Some(Track::CornerBackSlash),
                    cart: None,
                }),
                '^' => Some(MapPosition {
                    position: position.clone(),
                    track: Some(Track::Vertical),
                    cart: Some(Cart {
                        position,
                        direction: CartDirection::Up,
                        next_turn: NextTurn::Left
                    }),
                }),
                'v' => Some(MapPosition {
                    position: position.clone(),
                    track: Some(Track::Vertical),
                    cart: Some(Cart {
                        position,
                        direction: CartDirection::Down,
                        next_turn: NextTurn::Left
                    }),
                }),
                '>' => Some(MapPosition {
                    position: position.clone(),
                    track: Some(Track::Horizontal),
                    cart: Some(Cart {
                        position,
                        direction: CartDirection::Right,
                        next_turn: NextTurn::Left
                    }),
                }),
                '<' => Some(MapPosition {
                    position: position.clone(),
                    track: Some(Track::Horizontal),
                    cart: Some(Cart {
                        position,
                        direction: CartDirection::Left,
                        next_turn: NextTurn::Left
                    }),
                }),
                _ => None,
            }
        })
        .fold(
            (
                HashMap::new(),
                Vec::new(),
            ),
            |(mut track_map, mut carts), map_position| {
                if let Some(track) = map_position.track {
                    track_map.insert(map_position.position, track);
                }

                if let Some(cart) = map_position.cart {
                    carts.push(cart);
                }
                (track_map, carts)
            }
        )
}

fn print_state(state: &WorldState) {
    let (x_min, y_min, x_max, y_max) = state.track_map.keys().fold(
        (
            std::usize::MAX,
            std::usize::MAX,
            std::usize::MIN,
            std::usize::MIN,
        ),
        |(x_min, y_min, x_max, y_max), point| {
            (
                usize::min(x_min, point.x),
                usize::min(y_min, point.y),
                usize::max(x_max, point.x),
                usize::max(y_max, point.y),
            )
        },
    );
    for y in y_min..y_max + 1 {
        for x in x_min..x_max + 1 {
            let point = Point { x, y };
            let mut char_to_print = None;
            for (_cart_id, cart) in &state.carts {
                if cart.position == point {
                    match cart.direction {
                        CartDirection::Up => char_to_print = Some("^"),
                        CartDirection::Down => char_to_print = Some("v"),
                        CartDirection::Left => char_to_print = Some("<"),
                        CartDirection::Right => char_to_print = Some(">"),
                    }
                }
            }
            if char_to_print == None {
                match state.track_map.get(&Point { x, y }) {
                    Some(Track::Horizontal) => char_to_print = Some("-"),
                    Some(Track::Vertical) => char_to_print = Some("|"),
                    Some(Track::Intersection) => char_to_print = Some("+"),
                    Some(Track::CornerForwardSlash) => char_to_print = Some("/"),
                    Some(Track::CornerBackSlash) => char_to_print = Some("\\"),
                    None => char_to_print = Some(" "),
                }
            }
            print!("{}", char_to_print.unwrap());
        }
        print!("\n");
    }
}
