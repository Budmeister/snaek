
use std::{
    sync::{Arc, RwLock, mpsc::{Receiver, TryRecvError}},
    thread,
    time
};

use piston_window::Key;
use rand::Rng;

use super::types::{Board, Coord, CellState, Dir, BoardExt, B_HEIGHT, B_WIDTH, GameState, Snake};

use super::art::BoardArtExt;

pub const TIMER_RESET: usize = 100;
pub const POWERUP_RESET: usize = 100;
pub const POWERUP_FREEZE_RESET: usize = 10;

pub const MIN_LEN_FOR_FOOD_IN_SCOREBOARD: usize = 10;

pub fn reset() -> (Board, GameState) {
    let mut board = Board::new();
    let snake = Snake::new((10, 5), Dir::Right, 5);
    let food = (15, 5);
    place_snake_and_food(&mut board, &snake, food);
    draw_embelishments(&mut board);

    let state = GameState {
        snake,
        snake_len: 5,
        food: food.into(),
        powerup: None,
        timer: TIMER_RESET,
        powerup_strength: 0,
        powerup_freeze: POWERUP_FREEZE_RESET,
        failed: false,
    };

    (board, state)
}

pub fn spawn_logic_thread(board: Arc<RwLock<Board>>, mut s: GameState, rx: Receiver<Key>) -> thread::JoinHandle<()> {

    // Poll the Lazy
    crate::text::GRIDS.len();

    thread::spawn(move || {
        loop {
            {
                let mut board_w = board.write().unwrap();
                
                let poisoned = handle_keys(&rx, &mut board_w, &mut s);
                if poisoned {
                    return;
                }

                advance_board(&mut board_w, &mut s);
            }

            thread::sleep(time::Duration::from_millis(1000 / s.snake_len as u64));
        }
    })
}

// Returns true if Tx closed
fn handle_keys(rx: &Receiver<Key>, board: &mut Board, s: &mut GameState) -> bool {
    match rx.try_recv() {
        Ok(key) => {
            match key {
                Key::W | Key::Up => {
                    if !s.failed && s.powerup_freeze == 0 {
                        s.snake.point(Dir::Up)
                    }
                }
                Key::A | Key::Left => {
                    if !s.failed && s.powerup_freeze == 0 {
                        s.snake.point(Dir::Left)
                    }
                }
                Key::S | Key::Down => {
                    if !s.failed && s.powerup_freeze == 0 {
                        s.snake.point(Dir::Down)
                    }
                }
                Key::D | Key::Right => {
                    if !s.failed && s.powerup_freeze == 0 {
                        s.snake.point(Dir::Right)
                    }
                }
                Key::R => (*board, *s) = reset(),
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
fn advance_board(board: &mut Board, s: &mut GameState) {
    if s.failed {
        return;
    }

    if s.powerup_freeze != 0 {
        s.powerup_freeze -= 1;
        return;
    }

    let Coord {x: tail_x, y: tail_y } = s.snake.tail_pos();
    board[tail_y][tail_x] = CellState::Empty;

    // Advance snake
    let hit_edge = s.snake.advance(s.powerup_strength != 0);
    if hit_edge {
        println!("Hit edge! Failing.");
        fail(board, s);
        return;
    }
    update_embelishments(board, s);

    // Check what we hit
    let head_pos = s.snake.head_pos();
    if head_pos == s.food {
        println!("1 food gained. Score: {}", s.snake_len);
        s.snake.add_food(1);
        s.snake_len += 1;
        while board.cell_at(s.food) == CellState::Filled {
            if s.snake_len < MIN_LEN_FOR_FOOD_IN_SCOREBOARD {
                s.food = Coord::rand_range(0..E_START, 0..B_HEIGHT);
            } else {
                s.food = Coord::rand();
            }
        }
        board.pt(s.food);
    } else if Some(head_pos) == s.powerup {
        println!("Powerup gained for {} frames", POWERUP_RESET);
        s.powerup_strength = POWERUP_RESET;
        s.powerup = None;
    } else {
        match board.cell_at(head_pos) {
            CellState::Empty => {}
            CellState::Filled if s.powerup_strength != 0 => {}
            CellState::Filled => {
                println!("Hit filled cell! Failing.");
                fail(board, s);
                return;
            }
        }
    }

    // Decrement powerup
    if s.powerup_strength != 0 {
        s.powerup_strength -= 1;
    }

    // Decrement timer
    if s.timer == 0 {
        // Respawn the powerup
        if let Some(powerup) = s.powerup {
            board.pt_off(powerup);
        }
        let powerup = Coord::rand();
        let radius = rand::thread_rng().gen_range(5..35);

        board.circle(powerup, radius);
        draw_scoreboard_border(board);
        board.line_off((if radius < powerup.x { powerup.x - radius } else { 0 }, powerup.y), (powerup.x + radius, powerup.y));
        board.pt(powerup);

        s.powerup = Some(powerup);
        s.powerup_freeze = POWERUP_FREEZE_RESET;
        s.timer = TIMER_RESET;
    } else {
        s.timer -= 1;
    }

    board.pt(head_pos);
}

pub fn place_snake_and_food(board: &mut Board, snake: &Snake, food: impl Into<Coord>) {
    for ((c1, _), (c2, d2)) in snake.joints().iter().zip(snake.joints().iter().skip(1)) {
        let mut c2 = *c2;
        while c2 != *c1 {
            let Coord { x, y } = c2;
            board[y][x] = CellState::Filled;
            c2 = c2.add(*d2).expect("Couldn't place snake");
        }
        let Coord { x, y } = c2;
        board[y][x] = CellState::Filled;
    }
    board.set_cell_at(food.into(), CellState::Filled);
}

fn fail(_board: &mut Board, s: &mut GameState) {
    s.failed = true;
    println!("Failed");
}

const E_WIDTH: usize = 30;
const E_START: usize = B_WIDTH - E_WIDTH;
pub fn draw_embelishments(board: &mut Board) {
    draw_scoreboard_border(board);

    board.text("Snaek", (E_START + 5, 3));

    board.text("Timer:", (E_START + 2, 10));

    board.text("Head:", (E_START + 2, 27));
    
    board.text("Food:", (E_START + 2, 39));
}

pub fn update_embelishments(board: &mut Board, s: &GameState) {
    // Timer
    board.text(&format!("{}  ", s.timer), (E_START + 2, 16));
    // Head
    board.text(&format!("({},{}) ", s.snake.head_pos().x, s.snake.head_pos().y), (E_START + 2, 33));
    // Food
    board.text(&format!("({},{}) ", s.food.x, s.food.y), (E_START + 2, 45));
    if s.powerup_strength == 0 {
        // Length
        board.text(&format!("Len:{} ", s.snake_len), (E_START + 2, 51));
    } else {
        // Powerup
        board.text(&format!("PWR:{} ", s.powerup_strength), (E_START + 2, 51));
    }
}

fn draw_scoreboard_border(board: &mut Board) {
    board.line((E_START, 1), (E_START, B_HEIGHT - 2));
    board.pt((E_START + 1, 0));
    board.pt((E_START + 1, B_HEIGHT - 1));
}

