
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
    SeedableRng,
};

use super::{
    types::{
        Board,
        Dir,
        GameState,
        Snake,
        CellObject,
        CellState,
        CellFloor,
        Coord,
        MAX_WATER_DIST,
        SnakeColor,
        B_WIDTH,
        board_ops,
        B_HEIGHT,
        LOGIC_MAX_MSPT,
        DebugInfo,
        MAX_WATER_DIST_FOR_SEED_SPREAD,
        NUM_SHOP_ITEMS,
        ShopItem,
    },
    levels::{
        LEVELS,
        LevelState
    },
    scoreboard::{SCORE_BANNER_VERT, ScoreboardArt, ShopItemFill}, art::{PlusWater, PlusSeed},
};

use super::art::BoardArt;

pub const INVINC_TIME: usize = 100;

pub fn reset() -> (GameState, Box<dyn LevelState>) {
    let current_level_index = 0;
    let level = &LEVELS[current_level_index];
    let mut l = (level.new_level_state)();
    let shop = l.new_shop();
    println!("Level 1: {}", level.name);
    let mut board = Board::from_bytes(level.raw_board);
    let mut scoreboard = Board::from_bytes(SCORE_BANNER_VERT);
    scoreboard.shop_fill(&shop);
    let snake = Snake::new((5, 5), Dir::Right, 5);

    // _place_debug(&mut board);

    let s = GameState {
        level,
        scoreboard,
        shop,
        board,
        snake,
        coins: 300,
        invinc_time: 0,
        failed: false,
        frame_num: 0,
        debug_screen: false,
        debug_info: DebugInfo::default(),
        salt: rand::thread_rng().gen(),
    };

    (s, l)
}

const NUM_BOARD_ADVANCE_THREADS: u32 = 4;

///////////////////////////////////////////////////////////
pub fn spawn_logic_thread(s: Arc<RwLock<GameState>>, mut l: Box<dyn LevelState>, rx: Receiver<UserAction>) -> thread::JoinHandle<()> {
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
                
                let poisoned = handle_keys(&rx, &mut s_w, &mut l);
                if poisoned {
                    return;
                }

                advance_board(&mut s_w, &mut *l, &mut pool);

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
fn handle_keys(rx: &Receiver<UserAction>, s: &mut GameState, l: &mut Box<dyn LevelState>) -> bool {
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
                UserAction::ShopItem(shop_item_num) => {
                    set_shop_item_selected(s, shop_item_num);
                }
                UserAction::Buy => {
                    buy(s, &mut **l);
                }
                UserAction::Restart => if let Some(level_state) = s.next_level() {
                    *l = level_state
                },
                UserAction::Debug => {
                    s.debug_screen = !s.debug_screen;
                }
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

fn set_shop_item_selected(s: &mut GameState, shop_item_num: usize) {
    if shop_item_num > NUM_SHOP_ITEMS {
        return;
    }
    s.scoreboard.shop_item_display(&s.shop, s.shop.selected, ShopItemFill::Clear);
    s.shop.selected = shop_item_num;
    s.scoreboard.shop_item_display(&s.shop, s.shop.selected, ShopItemFill::FromItem);
}

fn buy(s: &mut GameState, l: &mut dyn LevelState) {
    let ShopItem { kind, mut price } = s.shop.get_selected();
    price *= s.shop.price_multiplier;
    
    println!("Buy! Current coins: {}. Cost: {}", s.coins, price);
    if price > s.coins {
        return;
    }
    s.coins -= price;

    match kind {
        super::types::PowerupType::Water => {
            s.board.explosion(s.snake.head_pos(), PlusWater(1));
        },
        super::types::PowerupType::Explosive => {
            println!("Explosive not implemented yet. Doing nothing.");
        },
        super::types::PowerupType::Shovel => {
            println!("Shovel not implemented yet. Doing nothing.");
        },
        super::types::PowerupType::Seed => {
            s.board.explosion(s.snake.head_pos(), PlusSeed(1));
        },
        super::types::PowerupType::Invincibility => {
            s.invinc_time += INVINC_TIME;
        },
    }
    
    s.scoreboard.shop_remove(&s.shop);
    l.reset_shop(s);
    s.scoreboard.shop_fill(&s.shop);
}

// Returns true if failed
fn advance_board(s: &mut GameState, l: &mut dyn LevelState, pool: &mut Pool) {
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

    l.update(s);

    // Decrement powerup
    if s.invinc_time != 0 {
        s.invinc_time -= 1;
        if s.invinc_time == 0 {
            println!("Invincibility over");
        }
    }

    s.frame_num += 1;

    s.board.pt(head_pos, CellObject::Snake(super::types::SnakeColor::Head, s.snake.len()));
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
        // Fail conditions - handled above - put last so that you can still get powerup on lava
        CellState { floor: CellFloor::Lava { .. }, .. } |
        CellState { obj: CellObject::Wall | CellObject::Border, .. } => {}
    }
}

pub fn tick(old_cell: &CellState, old_surrounding: [&CellState; 8], new_cell: &mut CellState, coord: Coord, s: &GameState) {
    tick_floor(old_cell, &old_surrounding, new_cell, coord, s);
    tick_object(&new_cell.clone(), &old_surrounding, new_cell, coord, s);
    liquid_flow(&new_cell.clone(), &old_surrounding, new_cell, coord, s);
}

fn tick_floor(old_cell: &CellState, old_surrounding: &[&CellState; 8], new_cell: &mut CellState, _coord: Coord, _s: &GameState) {
    match old_cell.floor {
        CellFloor::Empty | CellFloor::Indicator(..) => {
            // Find a cell that wants to spread to this cell
            let dist = old_surrounding
                .iter()
                .find_map(|cell| {
                    if let CellFloor::Seed { dist, .. } = cell.floor {
                        let new_dist = dist + 1;
                        if new_dist <= MAX_WATER_DIST_FOR_SEED_SPREAD {
                            return Some(new_dist);
                        }
                    }
                    None
                });
            // At least one seed would like to spread to this cell
            if dist.is_some() && rand::thread_rng().gen_range(0..100) == 0 {
                // max speed for dist is 1, so start at MAX_WATER_DIST and decrease from there
                new_cell.update(CellFloor::Seed { height: 1, dist: MAX_WATER_DIST_FOR_SEED_SPREAD });
            }
        }
        CellFloor::Water { .. } => {
        }
        CellFloor::Lava { .. } => {
        }
        CellFloor::Seed { height, dist } => {
            #[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
            enum Delta {
                Decrement,
                NoChange,
                Increment,
                One,
            }
            let mut delta = Delta::Increment;
            for cell in old_surrounding {
                match cell.floor {
                    CellFloor::Seed { dist: other_dist, .. } => {
                        if other_dist < dist.saturating_sub(1) {
                            delta = Delta::Decrement;
                        } else if other_dist < dist {
                            delta = delta.min(Delta::NoChange);
                        }
                    }
                    CellFloor::Water { .. } => delta = Delta::One,
                    _ => {}
                }
            }
            // If None, then this cell should increment its dist
            let new_dist = match delta {
                Delta::Decrement => dist.saturating_sub(1),
                Delta::NoChange => dist,
                Delta::Increment => dist.saturating_add(1),
                Delta::One => 1,
            };

            let mut new_height = height;

            if new_dist <= MAX_WATER_DIST_FOR_SEED_SPREAD {
                if rand::thread_rng().gen_range(0..100) == 0 {
                    new_height = new_height.saturating_add(1);
                }
            } else {
                new_height = new_height.saturating_sub(1);
            }
            if new_dist > MAX_WATER_DIST {
                new_cell.update(CellFloor::Empty);
            } else {
                new_cell.update(CellFloor::Seed { height: new_height, dist: new_dist });
            }
        }
    }
}


fn tick_object(old_cell: &CellState, _old_surrounding: &[&CellState; 8], new_cell: &mut CellState, _coord: Coord, _s: &GameState) {
    match old_cell.obj {
        CellObject::None => {}
        CellObject::Wall => {} // Conversion to water or lava is handled by CellFloor::Empty above
        CellObject::Snake(color, life) => {
            // println!("ticking snake {:?}", old_cell);
            if life >= 1 {
                let new_color = match color {
                    SnakeColor::Head | SnakeColor::LightRed => SnakeColor::DarkRed,
                    SnakeColor::DarkRed => SnakeColor::LightRed,
                };
                new_cell.update(CellObject::Snake(new_color, life - 1));
            } else {
                new_cell.update(CellObject::None);
            }
            // println!("new snake object: {:?}", new_cell);
        }
        CellObject::Food(life) => {
            if life >= 1 {
                new_cell.update(CellObject::Food(life - 1));
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
    
    if !can_participate_in_liquid_flow(old_cell) {
        return;
    }

    // If we are seed, and we are touching lava, become lava
    // This could destroy liquid that flows into this block on the same tick, but that's fine
    for cell in old_surrounding {
        if let (CellFloor::Seed { .. }, CellFloor::Lava { depth }) = (old_cell.floor, cell.floor) {
            new_cell.update(CellFloor::Lava { depth });
            return;
        }
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

    new_cell.update((floor, elev));
}

#[inline(always)]
fn can_participate_in_liquid_flow(cell: &CellState) -> bool {
    cell.obj != CellObject::Border
}

#[inline(always)]
fn can_liquid_flow(to: &CellState, from: &CellState) -> bool {
    if !can_participate_in_liquid_flow(to) || !can_participate_in_liquid_flow(from) {
        return false;
    }
    // Lava can always flow into seed, no matter how tall
    if matches!(to.floor, CellFloor::Seed { .. }) && matches!(from.floor, CellFloor::Lava { .. }) {
        return true;
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

    ShopItem(usize),
    Buy,

    Restart,
    Quit,
    Debug,
}
