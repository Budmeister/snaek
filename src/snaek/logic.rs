
use std::{
    sync::{Arc, RwLock, mpsc::{Receiver, TryRecvError}},
    thread,
    time::{Duration, Instant},
    hash::{Hash, Hasher},
    collections::hash_map::DefaultHasher,
};
use scoped_threadpool::Pool;

use rand::{
    Rng,
    distributions::{
        WeightedIndex,
        Distribution
    }, SeedableRng,
};

use crate::snaek::types::DebugInfo;

use super::{
    types::{
        Board,
        Dir,
        GameState,
        Snake,
        CellObject,
        CellState,
        CellFloor,
        PowerupType,
        Coord,
        MAX_WATER_DIST,
        SnakeColor,
        FOOD_AND_POWERUP_LIFETIME,
        B_WIDTH,
        board_ops,
        B_HEIGHT, LOGIC_MAX_MSPT,
    },
    levels
};

use super::art::BoardArt;

pub const TIMER_RESET: usize = 50;
pub const INVINC_TIME: usize = 100;

pub fn reset() -> GameState {
    println!("Level 1: {}", levels::LEVEL_NAMES[0]);
    let mut board = Board::from_bytes(levels::LEVELS[0]);
    let snake = Snake::new((5, 5), Dir::Right, 5);
    _place_debug(&mut board);

    // _place_debug(&mut board);

    GameState {
        current_level: 0,
        board,
        snake,
        timer: 0,
        invinc_time: 0,
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
        debug_screen: false,
        debug_info: DebugInfo::default(),
        salt: rand::thread_rng().gen(),
    }
}

fn next_level(s: &mut GameState) {
    s.current_level += 1;
    if s.current_level >= levels::NUM_LEVELS {
        return;
    }
    println!("Level {}: {}", s.current_level + 1, levels::LEVEL_NAMES[s.current_level]);
    s.board = Board::from_bytes(levels::LEVELS[s.current_level]);
    let snake = Snake::new((5, 5), Dir::Right, s.snake.len());
    
    s.snake = snake;
    s.timer = 0;
    s.failed = false;
}

const NUM_BOARD_ADVANCE_THREADS: u32 = 4;

///////////////////////////////////////////////////////////
pub fn spawn_logic_thread(s: Arc<RwLock<GameState>>, rx: Receiver<UserAction>) -> thread::JoinHandle<()> {
    // Poll the Lazy
    crate::text::GRIDS.len();

    thread::spawn(move || {
        let mut pool = Pool::new(NUM_BOARD_ADVANCE_THREADS);
        loop {
            let start = Instant::now();

            let processing_time;
            {
                let lock_start = start.elapsed();
                let mut s_w = s.write().unwrap();
                let lock_gotten = start.elapsed();
                let lock_time = lock_gotten - lock_start;
                
                let poisoned = handle_keys(&rx, &mut s_w);
                if poisoned {
                    return;
                }

                advance_board(&mut s_w, &mut pool);

                processing_time = start.elapsed();
                s_w.debug_info.lock_uspt = lock_time.as_micros();
                s_w.debug_info.proc_uspt = processing_time.as_micros();
            }

            // Write the duration to the file
            if let Some(remaining) = Duration::from_millis(LOGIC_MAX_MSPT).checked_sub(processing_time) {
                thread::sleep(remaining);
            } else {
                thread::sleep(Duration::from_millis(1));
            }
        }
    })
}
///////////////////////////////////////////////////////////

// Returns true if Tx closed
fn handle_keys(rx: &Receiver<UserAction>, s: &mut GameState) -> bool {
    match rx.try_recv() {
        Ok(key) => {
            match key {
                UserAction::Up => {
                    if !s.failed {
                        s.snake.point(Dir::Up)
                    }
                }
                UserAction::Left => {
                    if !s.failed {
                        s.snake.point(Dir::Left)
                    }
                }
                UserAction::Down => {
                    if !s.failed {
                        s.snake.point(Dir::Down)
                    }
                }
                UserAction::Right => {
                    if !s.failed {
                        s.snake.point(Dir::Right)
                    }
                }
                UserAction::Water => {
                    if !s.failed && s.water_pwrs != 0 {
                        s.water_pwrs -= 1;
                        s.board.explosion(s.snake.head_pos(), CellFloor::Water { depth: 1 });
                        println!("Used water powerup, {} remaining", s.water_pwrs);
                    }
                }
                UserAction::Explosion => {
                    if !s.failed && s.explo_pwrs != 0 {
                        s.explo_pwrs -= 1;
                        s.board.explosion(s.snake.head_pos(), (CellFloor::Empty, CellObject::None));
                        println!("Used explosion powerup, {} remaining", s.explo_pwrs);
                    }
                }
                UserAction::Turf => {
                    if !s.failed && s.turf_pwrs != 0 {
                        s.turf_pwrs -= 1;
                        s.board.explosion(s.snake.head_pos(), CellFloor::Turf);
                        println!("Used turf powerup, {} remaining", s.turf_pwrs);
                    }
                }
                UserAction::Seed => {
                    if !s.failed && s.seed_pwrs != 0 {
                        s.seed_pwrs -= 1;
                        s.board.explosion(s.snake.head_pos(), CellFloor::Seed(MAX_WATER_DIST));
                        println!("Used seed powerup, {} remaining", s.seed_pwrs);
                    }
                }
                UserAction::Restart => next_level(s),
                UserAction::Debug => {
                    s.debug_screen = !s.debug_screen;
                }
                UserAction::Shop => {}
                UserAction::Quit => {}
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
fn advance_board(s: &mut GameState, pool: &mut Pool) {
    if s.failed {
        return;
    }

    // Advance snake
    s.snake.advance();

    // Check what we hit
    let head_pos = s.snake.head_pos();
    handle_hit(s.board.cell_at(head_pos), s);
    s.board.pt(head_pos, CellObject::Wall);

    // Update all cells
    let mut board_new = s.board.clone();
    let mut board_new_slice: &mut [[CellState; B_WIDTH]] = &mut board_new[1..B_HEIGHT - 1];
    let mut slices = Vec::new();
    let mut slices_left = NUM_BOARD_ADVANCE_THREADS as usize;
    let mut start = 1;
    for _ in 0..NUM_BOARD_ADVANCE_THREADS {
        // Calculate size for new_slice
        let size = board_new_slice.len() / slices_left;
        let (new_slice, next) = board_new_slice.split_at_mut(size);

        // Calculate size for old_slice
        let end = usize::min(start + size, s.board.len() - 1);
        let old_slice = &s.board[start-1..end+1];

        slices.push((old_slice, new_slice, start));
        board_new_slice = next;
        slices_left -= 1;
        start = end;
    }
    pool.scoped(|scope| {
        for (old_slice, new_slice, start_at_y) in slices {
            scope.execute({
            let s = &s;
            move || {
                let iter = board_ops::surrounding(old_slice)
                        .zip(board_ops::inner_cells_horiz_mut_enumerate(new_slice, start_at_y));
                for ((old_cell, old_surrounding), (coord, new_cell)) in iter {
                    tick(old_cell, old_surrounding, new_cell, coord, s);
                }
            }});
        }
    });
    s.board = board_new;

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
        s.board.pt(pwr_coords, CellObject::Powerup(pwr, FOOD_AND_POWERUP_LIFETIME));

        let food_coords: Coord = rand::random();
        s.board.pt(food_coords, CellObject::Food(FOOD_AND_POWERUP_LIFETIME));

        // ...

        // s.powerup_freeze = POWERUP_FREEZE_RESET;
        s.timer = TIMER_RESET;
    } else {
        s.timer -= 1;
    }
    s.frame_num += 1;

    s.board.pt(head_pos, CellObject::Snake(super::types::SnakeColor::Head, s.snake.len()));
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
        CellState { floor: CellFloor::Lava { .. }, .. } |
        CellState { obj: CellObject::Wall, .. } => {
            if s.invinc_time == 0 {
                fail(s, "Hit wall or lava!");
            }
        }
        CellState { obj: CellObject::Border, .. } => {
            // Fail even with invincibility
            fail(s, "Hit border!");
        }
        _ => {}
    }
    match cell {
        // Safe conditions
        CellState { obj: CellObject::None | CellObject::Snake(_, _), .. } => {}
        CellState { obj: CellObject::Food(..), .. } => {
            println!("Ate food. Score: {}", s.snake.len());
            s.snake.add_food(1);
        }
        CellState { obj: CellObject::Powerup(pwr, _), .. } => {
            println!("Got powerup: {:?}", pwr);
            match pwr {
                PowerupType::Water          => s.water_pwrs += 1,
                PowerupType::Explosive      => s.explo_pwrs += 1,
                PowerupType::Turf           => s.turf_pwrs += 1,
                PowerupType::Seed           => s.seed_pwrs += 1,
                PowerupType::Invincibility  => s.invinc_time = INVINC_TIME,
            }
        }
        CellState { obj: CellObject::SuperPowerup(pwr, _), .. } => {
            println!("Got super-powerup: {:?}", pwr);
            match pwr {
                PowerupType::Water          => s.water_pwrs += 10,
                PowerupType::Explosive      => s.explo_pwrs += 10,
                PowerupType::Turf           => s.turf_pwrs += 10,
                PowerupType::Seed           => s.seed_pwrs += 10,
                PowerupType::Invincibility  => s.invinc_time = INVINC_TIME * 10,
            }
        }
        // Fail conditions - handled above - put last so that you can still get powerup on lava
        CellState { floor: CellFloor::Lava { .. }, .. } |
        CellState { obj: CellObject::Wall | CellObject::Border, .. } => {}
    }
    // Update counts
    match cell.floor {
        CellFloor::Empty => s.empty_count += 1,
        CellFloor::Water { .. } => s.water_count += 1,
        CellFloor::Lava { .. } => s.lava_count += 1,
        CellFloor::Turf => s.turf_count += 1,
        CellFloor::Seed(_) => s.seed_count += 1,
        CellFloor::DeadSeed | CellFloor::Indicator(..) => {},
    }
}

macro_rules! count_matches {
    // The macro will take an array and a list of items to match.
    ($arr:expr, $( $x:pat ),*) => {{
        // Convert the array into an iterator.
        let iter = (&$arr).clone().into_iter();

        // Initialize a tuple to store counts. We use a tuple here assuming a known number of items.
        let counts = ( $( iter.clone().filter(|&item| match item { $x => true, _ => false }).count(), )* );

        // Return the counts.
        counts
    }};
}

pub fn tick(old_cell: &CellState, old_surrounding: [&CellState; 8], new_cell: &mut CellState, coord: Coord, s: &GameState) {
    tick_floor(old_cell, &old_surrounding, new_cell, coord, s);
    tick_object(old_cell, &old_surrounding, new_cell, coord, s);
    liquid_flow(old_cell, &old_surrounding, new_cell, coord, s);
}

fn tick_floor(old_cell: &CellState, old_surrounding: &[&CellState; 8], new_cell: &mut CellState, _coord: Coord, _s: &GameState) {
    match old_cell.floor {
        CellFloor::Turf => {}
        CellFloor::Empty | CellFloor::Indicator(..) => {
            let (s,) = count_matches!(old_surrounding.iter().map(|state| state.floor), CellFloor::Seed(_));
            let weights = [
                3_00 * s + 1,       // Seed spreads to this block
                2_000_000           // Nothing happens
            ];
            let dist = WeightedIndex::new(&weights).unwrap();

            let mut rng = rand::thread_rng();
            match dist.sample(&mut rng) {
                2 => new_cell.update(CellFloor::Seed(0)),
                3 => {}
                _ => {}
            }
        }
        CellFloor::Water { .. } => {
        }
        CellFloor::Lava { .. } => {
        }
        CellFloor::Seed(dist) => {
            let mut rng = rand::thread_rng();
            let dist_inv = MAX_WATER_DIST.saturating_sub(dist);
            let weights = [
                2 * dist,       // This seed dies
                2 * dist_inv,   // This seed spawns food
                700             // Nothing happens
            ];
            let distr = WeightedIndex::new(&weights).unwrap();

            let num = distr.sample(&mut rng);
            match num {
                2 => new_cell.update((CellFloor::DeadSeed, CellObject::None)),
                3 => new_cell.update(CellObject::Food(FOOD_AND_POWERUP_LIFETIME)),
                4 => {}
                _ => {}
            }
            // It's still a seed, so update the dist from water
            if num == 3 || num == 4 {
                let dist_new = old_surrounding
                    .iter()
                    .filter_map(|x| match x.floor {
                        CellFloor::Seed(dist) => Some(dist),
                        CellFloor::Water { .. } => Some(0),
                        _ => None,
                    })
                    .min()
                    .unwrap_or(MAX_WATER_DIST) + 1;
                new_cell.update(CellFloor::Seed(dist_new));
            }
        }
        CellFloor::DeadSeed => {
            let (s,) = count_matches!(old_surrounding.iter().map(|state| state.floor), CellFloor::Seed(_));
            let mut rng = rand::thread_rng();
            let weights = [
                3 * s,      // Seed spreads to this block
                1,          // This dead seed despawns
                3_500       // Nothing happens
            ];
            let distr = WeightedIndex::new(&weights).unwrap();

            let num = distr.sample(&mut rng);
            match num {
                2 => new_cell.update(CellFloor::Seed(0)),
                3 => new_cell.update(CellFloor::Empty),
                4 => {}
                _ => {}
            }
        }
    }
}


fn tick_object(old_cell: &CellState, _old_surrounding: &[&CellState; 8], new_cell: &mut CellState, _coord: Coord, _s: &GameState) {
    match old_cell.obj {
        CellObject::None => {}
        CellObject::Wall => {} // Conversion to water or lava is handled by CellFloor::Empty above
        CellObject::Snake(color, life) => {
            if life >= 1 {
                let new_color = match color {
                    SnakeColor::Head | SnakeColor::LightRed => SnakeColor::DarkRed,
                    SnakeColor::DarkRed => SnakeColor::LightRed,
                };
                new_cell.update(CellObject::Snake(new_color, life - 1));
            } else {
                new_cell.update(CellObject::None);
            }
        }
        CellObject::Food(life) => {
            if life >= 1 {
                new_cell.update(CellObject::Food(life - 1));
            } else {
                new_cell.update(CellObject::None);
            }
        }
        CellObject::Powerup(pwr, life) => {
            if life >= 1 {
                new_cell.update(CellObject::Powerup(pwr, life - 1));
            } else {
                new_cell.update(CellObject::None);
            }
        }
        CellObject::SuperPowerup(pwr, life) => {
            if life >= 1 {
                new_cell.update(CellObject::SuperPowerup(pwr, life - 1));
            } else {
                new_cell.update(CellObject::None);
            }
        }
        CellObject::Border => {}
    }
}

const SURROUNDING_COORDS: [(isize, isize); 8] = [
    (-1, -1), ( 0, -1), ( 1, -1),
    (-1,  0),           ( 1,  0),
    ( 1, -1), ( 0,  1), ( 1,  1),
];

fn liquid_flow(old_cell: &CellState, old_surrounding: &[&CellState; 8], new_cell: &mut CellState, coord: Coord, s: &GameState) {
    // To give liquid
    // 1. For this cell, create a rng seeded with the hash of (coord, frame num, salt). 
    // 2. Generate a number from 1-8
    // 3. That number is the index of the surrounding cell that receives the liquid, if it can. Only if it can, then
    //    decrease liquid in this cell by one.
    // 
    // To receive liquid
    // 1. For each surrounding cell, create a rng based on the info for that cell
    // 2. Figure out if that cell would have given liquid to this cell
    // 3. Count up all the liquid gained

    // Giving takes place before receiving
    
    if !can_participate_in_liquid_flow(old_cell) {
        return;
    }

    // Give liquid
    let mut floor = old_cell.floor; // make mutable copy
    if let CellFloor::Water { depth } | CellFloor::Lava { depth } = &mut floor {
        let mut self_rng = get_local_rng(coord, s);
        let to = self_rng.gen_range(0..8usize);
        let to = old_surrounding[to];
        if can_liquid_flow(to, old_cell) {
            // We can give one away, so minus one
            *depth -= 1;
        }
    }

    // 0 1 2      7 6 5
    // 3   4  =>  4   3
    // 5 6 7      2 1 0

    // To reverse, just do 7 - i

    // Receive liquid
    let mut water: u8 = 0;
    let mut lava: u8 = 0;
    let Coord { x, y } = coord;

    for (i, (from, coord)) in old_surrounding
            .iter()
            .zip(SURROUNDING_COORDS.map(|(dx, dy)| {
                Coord { x: (x as isize + dx) as usize, y: (y as isize + dy) as usize }
            }))
            .enumerate()
    {
        if !can_liquid_flow(old_cell, from) {
            continue;
        }
        if let (liquid, _, CellFloor::Water { .. }) | (_, liquid, CellFloor::Lava { .. }) = (&mut water, &mut lava, from.floor) {
            let mut giver_rng = get_local_rng(coord, s);
            let to_receive = giver_rng.gen_range(0..8usize);
            if to_receive == 7 - i {
                *liquid += 1;
            }
        }
    }

    if let (liquid, _, CellFloor::Water { depth }) | (_, liquid, CellFloor::Lava { depth }) = (&mut water, &mut lava, floor) {
        *liquid = liquid.saturating_add(depth)
    }

    let elev;
    match water.cmp(&lava) {
        std::cmp::Ordering::Less => {
            elev = old_cell.elev.saturating_add(water);
            floor = CellFloor::Lava { depth: lava - water };
        },
        std::cmp::Ordering::Equal => {
            elev = old_cell.elev.saturating_add(water);
            // If this cell was water or lava, it is now empty
            // otherwise, it just keeps its state
            if let CellFloor::Water { .. } | CellFloor::Lava { .. } = old_cell.floor {
                floor = CellFloor::Empty;
            } else {
                floor = old_cell.floor;
            }
        },
        std::cmp::Ordering::Greater => {
            elev = old_cell.elev.saturating_add(lava);
            floor = CellFloor::Water { depth: water - lava };
        },
    }

    *new_cell = CellState { floor, obj: old_cell.obj, elev };
}

#[inline(always)]
fn can_participate_in_liquid_flow(cell: &CellState) -> bool {
    cell.floor != CellFloor::Turf && cell.obj != CellObject::Border
}

#[inline(always)]
fn can_liquid_flow(to: &CellState, from: &CellState) -> bool {
    if !can_participate_in_liquid_flow(to) || !can_participate_in_liquid_flow(from) {
        return false;
    }
    to.roof() < from.roof()
}

#[inline(always)]
fn get_local_rng(coord: Coord, s: &GameState) -> impl Rng {
    let mut hasher = DefaultHasher::new();
    (coord, s.frame_num, s.salt).hash(&mut hasher);
    let hash = hasher.finish();
    rand::rngs::StdRng::seed_from_u64(hash)
}

/*
pub fn random_tick(cell: &CellState, surrounding: [&CellState; 8], cell_new: &mut CellState, mut surrounding_new: [&mut CellState; 8]) {
    let num = rand::thread_rng().gen_range(0..1_000_000);
    let i = rand::thread_rng().gen_range(0..8);
    match cell.floor {
        CellFloor::Turf | CellFloor::DeadSeed | CellFloor::ExplIndicator => {}
        CellFloor::Lava => {
            // Spread Lava
            let to_spread = CellState { floor: CellFloor::Lava, obj: CellObject::None };
            let spread_to = &mut surrounding_new[i];
            match spread_to {
                CellState { obj: CellObject::Border | CellObject::Snake(_, _), .. } => {}
                CellState { floor: CellFloor::Turf | CellFloor::Lava, .. } => {}
                CellState { floor: CellFloor::Seed(_) | CellFloor::DeadSeed, .. } => {
                    **spread_to = to_spread;
                }
                CellState { floor: CellFloor::Empty | CellFloor::ExplIndicator, obj: CellObject::None | CellObject::Powerup(..) | CellObject::SuperPowerup(..) | CellObject::Food(..) } => {
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
        CellFloor::Seed(dist) => {
            // Spread seed
            let to_spread = CellFloor::Seed(dist + 1);
            let spread_to = &mut surrounding_new[i];
            match spread_to {
                CellState { obj: CellObject::Border | CellObject::Wall | CellObject::Food(..) | CellObject::Snake(_, _), .. } => {}
                CellState { floor: CellFloor::Turf | CellFloor::Lava | CellFloor::Seed(_), ..} => {}
                CellState { floor: CellFloor::Water, obj: CellObject::None | CellObject::Powerup(..) | CellObject::SuperPowerup(..) } => {
                    // 1/50 chance of spreading
                    if num < 20_000 {
                        spread_to.update(to_spread);
                    }
                }
                CellState { floor: CellFloor::Empty | CellFloor::DeadSeed | CellFloor::ExplIndicator, obj: CellObject::None | CellObject::Powerup(..) | CellObject::SuperPowerup(..) } => {
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
            if !matches!(cell.obj, CellObject::Border | CellObject::Snake(_, _)) {
                let num = rand::thread_rng().gen_range(0..1_000_000);
                // 1/1000 chance to spawn food or die
                if num < 1_000 {
                    let num = rand::thread_rng().gen_range(0..(MAX_WATER_DIST * MAX_WATER_DIST));
                    // dist^2/MAX_WATER_DIST^2 chance to spawn die
                    if num < dist * dist {
                        cell_new.update(CellFloor::DeadSeed);
                    } else {
                        cell_new.update(CellObject::Food(FOOD_AND_POWERUP_LIFETIME));
                    }
                }
            }
        }
        CellFloor::Water => {
            // Spread water
            let to_spread = CellState { floor: CellFloor::Water, obj: CellObject::None };
            let spread_to = &mut surrounding_new[i];
            match spread_to {
                CellState { obj: CellObject::Border, .. } => {}
                CellState { floor: CellFloor::Turf | CellFloor::Water | CellFloor::Seed(_), .. } => {}
                CellState { floor: CellFloor::Empty | CellFloor::DeadSeed | CellFloor::ExplIndicator, obj: CellObject::Wall | CellObject::Snake(..) } => {}
                CellState { floor: CellFloor::Empty | CellFloor::DeadSeed | CellFloor::ExplIndicator, obj: CellObject::None | CellObject::Powerup(..) | CellObject::SuperPowerup(..) | CellObject::Food(..) } => {
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
        CellFloor::Empty => {
            if cell.obj == CellObject::None {
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
        }
    }
    match cell.obj {
        CellObject::Snake(snake_color, life) => {
            if life == 0 {
                cell_new.update(CellObject::None);
                return;
            }
            let new_snake_color = match snake_color {
                SnakeColor::Head | SnakeColor::LightRed => SnakeColor::DarkRed,
                SnakeColor::DarkRed => SnakeColor::LightRed,
            };
            cell_new.update(CellObject::Snake(new_snake_color, life - 1));
        }
        _ => {}
    }
}
*/

fn _place_debug(board: &mut Board) {
    board.pt((10, 10), CellFloor::Water { depth: 200 });

    // for i in 2..=10 {
    //     board.pt((5 * i, 10), CellFloor::Empty);
    //     board.pt((5 * i, 15), CellFloor::Lava { depth: 1 });
    //     board.pt((5 * i, 20), CellFloor::Water { depth: 1 });
    //     board.pt((5 * i, 25), CellFloor::Turf);
    //     board.pt((5 * i, 30), CellFloor::Seed(MAX_WATER_DIST));
    // }
    
    // for i in 2..=6 {
    //     board.pt((10, 5 * i), CellObject::Border);
    //     board.pt((15, 5 * i), CellObject::Food(usize::MAX));
    //     board.pt((20, 5 * i), CellObject::None);
    //     board.pt((25, 5 * i), CellObject::Wall);
    //     board.pt((30, 5 * i), CellObject::Powerup(PowerupType::Water, usize::MAX));
    //     board.pt((35, 5 * i), CellObject::Powerup(PowerupType::Explosive, usize::MAX));
    //     board.pt((40, 5 * i), CellObject::Powerup(PowerupType::Turf, usize::MAX));
    //     board.pt((45, 5 * i), CellObject::Powerup(PowerupType::Invincibility, usize::MAX));
    //     board.pt((50, 5 * i), CellObject::Powerup(PowerupType::Seed, usize::MAX));
    // }
}

fn fail(s: &mut GameState, message: &str) {
    s.failed = true;
    println!("{}", message);
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
    Quit,
    Debug,
}
