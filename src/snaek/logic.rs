
use std::{
    sync::{Arc, RwLock, mpsc::{Receiver, TryRecvError}},
    thread,
    time
};

use piston_window::Key;
use rand::Rng;

use super::types::{Board, Coord, CellState, Dir, B_HEIGHT, B_WIDTH, GameState, Snake, CellObject};

use super::art::BoardArt;

pub const TIMER_RESET: usize = 100;
pub const POWERUP_RESET: usize = 100;
pub const POWERUP_FREEZE_RESET: usize = 10;

pub const MIN_LEN_FOR_FOOD_IN_SCOREBOARD: usize = 10;

pub fn reset() -> GameState {
    let mut board = Board::new();
    let snake = Snake::new((10, 5), Dir::Right, 5);
    place_snake(&mut board, &snake);

    let state = GameState {
        board,
        snake,
        snake_len: 5,
        timer: TIMER_RESET,
        invinc_time: 0,
        powerup_freeze: POWERUP_FREEZE_RESET,
        failed: false,
        frame_num: 0,
    };

    state
}

pub fn spawn_logic_thread(s: Arc<RwLock<GameState>>, rx: Receiver<Key>) -> thread::JoinHandle<()> {

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
fn handle_keys(rx: &Receiver<Key>, s: &mut GameState) -> bool {
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
                Key::R => *s = reset(),
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
    // ...

    // Decrement powerup
    if s.invinc_time != 0 {
        s.invinc_time -= 1;
    }

    // Decrement timer
    if s.timer == 0 {
        // Respawn the powerup

        // ...

        // s.powerup_freeze = POWERUP_FREEZE_RESET;
        s.timer = TIMER_RESET;
    } else {
        s.timer -= 1;
    }

    s.board.pt(head_pos, CellObject::Wall);
}

pub fn place_snake(board: &mut Board, snake: &Snake) {
    for ((c1, _), (c2, d2)) in snake.joints().iter().zip(snake.joints().iter().skip(1)) {
        let mut c2 = *c2;
        while c2 != *c1 {
            board.pt(c2, CellObject::Wall);
            c2 = c2.add(*d2).expect("Couldn't place snake");
        }
        board.pt(c2, CellObject::Wall);
    }
}

fn fail(s: &mut GameState) {
    s.failed = true;
    println!("Failed");
}
