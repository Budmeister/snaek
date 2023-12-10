
use std::{
    sync::{Arc, RwLock, mpsc::{Receiver, TryRecvError}},
    thread,
    time
};

use rand::Rng;

use super::{types::{Board, Dir, GameState, Snake, CellObject, CellState, CellFloor, PowerupType, Coord, MAX_WATER_DIST, G_HEIGHT}, levels};

use super::art::BoardArt;

pub const TIMER_RESET: usize = 50;
pub const INVINC_TIME: usize = 100;
pub const POWERUP_FREEZE_RESET: usize = 10;

pub fn reset() -> GameState {
    println!("Level 1: {}", levels::LEVEL_NAMES[0]);
    let mut board = Board::from_bytes(levels::LEVELS[0]);
    let snake = Snake::new((10, 5), Dir::Right, 5);
    place_snake(&mut board, &snake);

    // _place_debug(&mut board);

    GameState {
        current_level: 0,
        board,
        snake,
        snake_len: 5,
        snake_color: false,
        timer: 0,
        invinc_time: 0,
        powerup_freeze: POWERUP_FREEZE_RESET,
        failed: false,
        frame_num: 0,
        water_pwrs: 0,
        explo_pwrs: 0,
        turf_pwrs: 0,
        seed_pwrs: 0,
        empty_count: 0,
        water_count: 0,
        lava_count: 0,
        turf_count: 0,
        seed_count: 0,
    }
}

fn next_level(s: &mut GameState) {
    s.current_level += 1;
    if s.current_level >= levels::NUM_LEVELS {
        return;
    }
    println!("Level {}: {}", s.current_level + 1, levels::LEVEL_NAMES[s.current_level]);
    s.board = Board::from_bytes(levels::LEVELS[s.current_level]);
    let mut snake = Snake::new((10, 5), Dir::Right, 5);
    snake.add_food(s.snake_len - 5);
    place_snake(&mut s.board, &snake);
    
    s.snake = snake;
    s.timer = 0;
    s.powerup_freeze = POWERUP_FREEZE_RESET;
    s.failed = false;
}

pub fn spawn_logic_thread(s: Arc<RwLock<GameState>>, rx: Receiver<UserAction>) -> thread::JoinHandle<()> {

    // Poll the Lazy
    crate::text::GRIDS.len();

    thread::spawn(move || {
        loop {
            {
                let mut s_w = s.write().unwrap();
                
                let poisoned = handle_keys(&rx, &mut s_w);
                if poisoned {
                    return;
                }

                advance_board(&mut s_w);
            }

            thread::sleep(time::Duration::from_millis(100));
        }
    })
}

// Returns true if Tx closed
fn handle_keys(rx: &Receiver<UserAction>, s: &mut GameState) -> bool {
    match rx.try_recv() {
        Ok(key) => {
            match key {
                UserAction::Up => {
                    if !s.failed && s.powerup_freeze == 0 {
                        s.snake.point(Dir::Up)
                    }
                }
                UserAction::Left => {
                    if !s.failed && s.powerup_freeze == 0 {
                        s.snake.point(Dir::Left)
                    }
                }
                UserAction::Down => {
                    if !s.failed && s.powerup_freeze == 0 {
                        s.snake.point(Dir::Down)
                    }
                }
                UserAction::Right => {
                    if !s.failed && s.powerup_freeze == 0 {
                        s.snake.point(Dir::Right)
                    }
                }
                UserAction::Water => {
                    if s.water_pwrs != 0 {
                        s.water_pwrs -= 1;
                        s.board.explosion(s.snake.head_pos(), CellFloor::Water);
                        println!("Used water powerup, {} remaining", s.water_pwrs);
                    }
                }
                UserAction::Explosion => {
                    if s.explo_pwrs != 0 {
                        s.explo_pwrs -= 1;
                        s.board.explosion(s.snake.head_pos(), CellState { floor: CellFloor::Empty, obj: CellObject::None });
                        println!("Used explosion powerup, {} remaining", s.explo_pwrs);
                    }
                }
                UserAction::Turf => {
                    if s.turf_pwrs != 0 {
                        s.turf_pwrs -= 1;
                        s.board.explosion(s.snake.head_pos(), CellFloor::Turf);
                        println!("Used turf powerup, {} remaining", s.turf_pwrs);
                    }
                }
                UserAction::Seed => {
                    if s.seed_pwrs != 0 {
                        s.seed_pwrs -= 1;
                        s.board.explosion(s.snake.head_pos(), CellFloor::Seed(MAX_WATER_DIST));
                        println!("Used seed powerup, {} remaining", s.seed_pwrs);
                    }
                }
                UserAction::Restart => next_level(s),
                _ => return false,
            };
        }
        Err(TryRecvError::Empty) => (),
        Err(TryRecvError::Disconnected) => {
            println!("Tx closed. Exiting thread.");
            return true;
        }
    }

    false
}

// Returns true if failed
fn advance_board(s: &mut GameState) {
    if s.failed {
        return;
    }

    if s.powerup_freeze != 0 {
        s.powerup_freeze -= 1;
        return;
    }

    s.board.pt(s.snake.tail_pos(), CellObject::None);

    // Advance snake
    let hit_edge = s.snake.advance(s.invinc_time != 0);
    if hit_edge {
        println!("Hit edge! Failing.");
        fail(s);
        return;
    }

    // Check what we hit
    let head_pos = s.snake.head_pos();
    handle_hit(s.board.cell_at(head_pos), s);
    s.board.pt(head_pos, CellObject::Wall);

    // Update all cells
    let mut board_new = s.board.clone();
    // We can't zip on surrounding, because it's not an iterator
    // Save a copy of the refs
    let mut board_refs = Vec::new();
    s.board.surrounding(|cell, surrounding| board_refs.push((cell, surrounding)));
    let mut old_iter = board_refs.into_iter();
    board_new.surrounding_mut(|new_cell, new_surrounding| {
        let (old_cell, old_surrounding) = old_iter.next().unwrap();
        random_tick(old_cell, old_surrounding, new_cell, new_surrounding);
    });
    s.board = board_new;

    place_score_banner_details(s);

    // Decrement powerup
    if s.invinc_time != 0 {
        s.invinc_time -= 1;
        if s.invinc_time == 0 {
            println!("Invincibility over");
        }
    }

    // Decrement timer
    if s.timer == 0 {
        // Respawn the powerup
        let pwr = choose_powerup_type(s);
        s.empty_count = 0;
        s.water_count = 0;
        s.lava_count = 0;
        s.turf_count = 0;
        let pwr_coords: Coord = rand::random();
        s.board.pt(pwr_coords, CellObject::Powerup(pwr));

        let food_coords: Coord = rand::random();
        s.board.pt(food_coords, CellObject::Food);

        // ...

        // s.powerup_freeze = POWERUP_FREEZE_RESET;
        s.timer = TIMER_RESET;
    } else {
        s.timer -= 1;
    }
    s.frame_num += 1;

    s.board.pt(head_pos, CellObject::Snake(s.snake_color));
    s.snake_color = !s.snake_color;
}

fn choose_powerup_type(s: &GameState) -> PowerupType {
    if s.current_level == 3 {
        // Only spawn seeds on level 3
        return PowerupType::Seed;
    }

    let sum = s.empty_count + s.water_count + s.lava_count + s.turf_count;
    if sum == 0 {
        return rand::random();
    }
    let mut num = rand::thread_rng().gen_range(0..sum);
    if num < s.empty_count {
        return rand::random();
    }
    num -= s.empty_count;
    if num < s.water_count {
        return PowerupType::Turf;
    }
    num -= s.water_count;
    if num < s.lava_count {
        return PowerupType::Seed;
    }
    num -= s.lava_count;
    // It's turf
    PowerupType::Water
}

fn handle_hit(cell: CellState, s: &mut GameState) {
    // Handle failing separately
    match cell {
        CellState { floor: CellFloor::Lava, .. } |
        CellState { obj: CellObject::Wall, .. } => {
            if s.invinc_time == 0 {
                fail(s);
            }
        }
        CellState { obj: CellObject::Border, .. } => {
            // Fail even with invincibility
            fail(s);
        }
        _ => {}
    }
    match cell {
        // Safe conditions
        CellState { floor: _, obj: CellObject::None | CellObject::Snake(_) } => {}
        CellState { floor: _, obj: CellObject::Food } => {
            s.snake_len += 1;
            println!("Ate food. Score: {}", s.snake_len);
            s.snake.add_food(1);
        }
        CellState { floor: _, obj: CellObject::Powerup(pwr) } => {
            println!("Got powerup: {:?}", pwr);
            match pwr {
                PowerupType::Water          => s.water_pwrs += 1,
                PowerupType::Explosive      => s.explo_pwrs += 1,
                PowerupType::Turf           => s.turf_pwrs += 1,
                PowerupType::Seed           => s.seed_pwrs += 1,
                PowerupType::Invincibility  => s.invinc_time = INVINC_TIME,
            }
        }
        // Fail conditions - handled above - put last so that you can still get powerup on lava
        CellState { floor: CellFloor::Lava, .. } |
        CellState { obj: CellObject::Wall | CellObject::Border, .. } => {}
    }
    // Update counts
    match cell.floor {
        CellFloor::Empty => s.empty_count += 1,
        CellFloor::Water => s.water_count += 1,
        CellFloor::Lava => s.lava_count += 1,
        CellFloor::Turf => s.turf_count += 1,
        CellFloor::Seed(_) => s.seed_count += 1,
        CellFloor::DeadSeed | CellFloor::ExplIndicator => {},
    }
}

pub fn random_tick(cell: &CellState, surrounding: [&CellState; 8], cell_new: &mut CellState, mut surrounding_new: [&mut CellState; 8]) {
    let num = rand::thread_rng().gen_range(0..1_000_000);
    let i = rand::thread_rng().gen_range(0..8);
    match cell {
        CellState { floor: CellFloor::Lava, obj: _ } => {
            // Spread Lava
            let to_spread = CellState { floor: CellFloor::Lava, obj: CellObject::None };
            let spread_to = &mut surrounding_new[i];
            match spread_to {
                CellState { obj: CellObject::Border | CellObject::Snake(_), .. } => {}
                CellState { floor: CellFloor::Turf | CellFloor::Lava, .. } => {}
                CellState { floor: CellFloor::Seed(_) | CellFloor::DeadSeed, .. } => {
                    **spread_to = to_spread;
                }
                CellState { floor: CellFloor::Empty | CellFloor::ExplIndicator, obj: CellObject::None | CellObject::Powerup(_) | CellObject::Food } => {
                    // 1/33 chance of spreading
                    if num < 30_000 {
                        **spread_to = to_spread;
                    }
                }
                CellState { floor: CellFloor::Empty | CellFloor::ExplIndicator, obj: CellObject::Wall } => {
                    // 1/100 chance of spreading
                    if num < 10_000 {
                        **spread_to = to_spread;
                    }
                }
                CellState { floor: CellFloor::Water, .. } => {
                    **spread_to = CellState { floor: CellFloor::Empty, obj: CellObject::Wall };
                }
            }
        }
        CellState { floor: CellFloor::Seed(dist), .. } => {
            // Spread seed
            let to_spread = CellFloor::Seed(*dist + 1);
            let spread_to = &mut surrounding_new[i];
            match spread_to {
                CellState { obj: CellObject::Border | CellObject::Wall | CellObject::Food | CellObject::Snake(_), .. } => {}
                CellState { floor: CellFloor::Turf | CellFloor::Lava | CellFloor::Seed(_), ..} => {}
                CellState { floor: CellFloor::Water, obj: CellObject::None | CellObject::Powerup(_) } => {
                    // 1/50 chance of spreading
                    if num < 20_000 {
                        spread_to.update(to_spread);
                    }
                }
                CellState { floor: CellFloor::Empty | CellFloor::DeadSeed | CellFloor::ExplIndicator, obj: CellObject::None | CellObject::Powerup(_) } => {
                    // 1/100 chance of spreading
                    if num < 5_000 {
                        spread_to.update(to_spread);
                    }
                }
            }
            let dist_new = surrounding
                .into_iter()
                .filter_map(|x| match x.floor {
                    CellFloor::Seed(dist) => Some(dist),
                    CellFloor::Water => Some(0),
                    _ => None,
                })
                .min()
                .unwrap_or(MAX_WATER_DIST) + 1;

            // The new cell could have been changed from being a seed. Only update dist if it is still a seed
            if let CellFloor::Seed(dist_new_) = &mut cell_new.floor {
                *dist_new_ = dist_new;
            }
            // Still calculate the effects as if it is a seed regardless
            if !matches!(cell.obj, CellObject::Border | CellObject::Snake(_)) {
                let num = rand::thread_rng().gen_range(0..1_000_000);
                // 1/1000 chance to spawn food or die
                if num < 1_000 {
                    let num = rand::thread_rng().gen_range(0..(MAX_WATER_DIST * MAX_WATER_DIST));
                    // dist^2/MAX_WATER_DIST^2 chance to spawn die
                    if num < dist * dist {
                        cell_new.update(CellFloor::DeadSeed);
                    } else {
                        cell_new.update(CellObject::Food);
                    }
                }
            }
        }
        CellState { floor: CellFloor::Water, obj: _ } => {
            // Spread water
            let to_spread = CellState { floor: CellFloor::Water, obj: CellObject::None };
            let spread_to = &mut surrounding_new[i];
            match spread_to {
                CellState { obj: CellObject::Border, .. } => {}
                CellState { floor: CellFloor::Turf | CellFloor::Water | CellFloor::Seed(_), .. } => {}
                CellState { floor: CellFloor::Empty | CellFloor::DeadSeed | CellFloor::ExplIndicator, obj: CellObject::Wall | CellObject::Snake(_) } => {}
                CellState { floor: CellFloor::Empty | CellFloor::DeadSeed | CellFloor::ExplIndicator, obj: CellObject::None | CellObject::Powerup(_) | CellObject::Food } => {
                    // 1/100 chance of spreading
                    if num < 5_000 {
                        **spread_to = to_spread;
                    }
                }
                CellState { floor: CellFloor::Lava, ..} => {
                    **spread_to = CellState { floor: CellFloor::Empty, obj: CellObject::Wall };
                }
            }
        }
        CellState { floor: CellFloor::Empty, obj: CellObject::None } => {
            // Spontaneously generate
            // 1/10_000
            if num < 1 {
                if i < 4 {
                    *cell_new = CellState { floor: CellFloor::Water, obj: CellObject::None };
                } else {
                    *cell_new = CellState { floor: CellFloor::Lava, obj: CellObject::None };
                }
            }
        }
        _ => {}
    }
}

pub fn place_snake(board: &mut Board, snake: &Snake) {
    let mut snake_color = true;
    for ((c1, _), (c2, d2)) in snake.joints().iter().zip(snake.joints().iter().skip(1)) {
        let mut c2 = *c2;
        while c2 != *c1 {
            board.pt(c2, CellObject::Snake(snake_color));
            c2 = c2.add(*d2).expect("Couldn't place snake");
            snake_color = !snake_color;
        }
        board.pt(c2, CellObject::Snake(snake_color));
    }
}

pub fn place_score_banner_details(s: &mut GameState) {
    let pwr_details = [
        (s.water_pwrs, CellState { floor: CellFloor::Water, obj: CellObject::None }),
        (s.explo_pwrs, CellState { floor: CellFloor::ExplIndicator, obj: CellObject::None }),
        (s.turf_pwrs, CellState { floor: CellFloor::Turf, obj: CellObject::None }),
        (s.seed_pwrs, CellState { floor: CellFloor::Seed(0), obj: CellObject::None })
    ];
    let y = 13 + G_HEIGHT;
    let empty = CellState { floor: CellFloor::Empty, obj: CellObject::None };

    for (i, (pwr_count, fill)) in pwr_details.into_iter().enumerate() {
        let x = 3 + 12 * i;
        let mut count = 0;

        for dy in 0..3 {
            for dx in 0..5 {
                let coord = (x + 2 * dx, y + 2 * dy);
                if count < pwr_count {
                    s.board.pt(coord, fill);
                } else {
                    s.board.pt(coord, empty);
                }
                count += 1;
            }
        }
    }
}

fn _place_debug(board: &mut Board) {
    for i in 2..=10 {
        board.pt((5 * i, 10), CellFloor::Empty);
        board.pt((5 * i, 15), CellFloor::Lava);
        board.pt((5 * i, 20), CellFloor::Water);
        board.pt((5 * i, 25), CellFloor::Turf);
        board.pt((5 * i, 30), CellFloor::Seed(MAX_WATER_DIST));
    }
    
    for i in 2..=6 {
        board.pt((10, 5 * i), CellObject::Border);
        board.pt((15, 5 * i), CellObject::Food);
        board.pt((20, 5 * i), CellObject::None);
        board.pt((25, 5 * i), CellObject::Wall);
        board.pt((30, 5 * i), CellObject::Powerup(PowerupType::Water));
        board.pt((35, 5 * i), CellObject::Powerup(PowerupType::Explosive));
        board.pt((40, 5 * i), CellObject::Powerup(PowerupType::Turf));
        board.pt((45, 5 * i), CellObject::Powerup(PowerupType::Invincibility));
        board.pt((50, 5 * i), CellObject::Powerup(PowerupType::Seed));
    }
}

fn fail(s: &mut GameState) {
    s.failed = true;
    println!("Failed. Press F to pay respects.");
}

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub enum UserAction {
    Up,
    Left,
    Down,
    Right,
    Water,
    Explosion,
    Turf,
    Seed,
    Shop,
    Restart,
}
