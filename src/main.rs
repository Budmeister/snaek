#![windows_subsystem = "windows"]

use std::sync::{Arc, RwLock, mpsc};

use draw::{create_window, window_loop};
use logic::spawn_logic_thread;


mod types;
mod draw;
mod logic;

fn main() {
    let (board, state) = logic::reset();
    
    let mut window = create_window();

    let board = Arc::new(RwLock::new(board));
    let (tx, rx) = mpsc::channel();
    spawn_logic_thread(board.clone(), state, rx);

    window_loop(&mut window, board, tx);
}
