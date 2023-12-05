#![windows_subsystem = "windows"]

use std::sync::{Arc, RwLock, mpsc};

use draw::create_window;
use piston_window::PistonWindow;

mod global;
mod draw;
mod classic;
mod snaek;
mod text;

fn main() {
    let window = create_window((global::W_WIDTH, global::W_HEIGHT));
    
    start_classic(window);
}


fn start_classic(mut window: PistonWindow) {
    let (board, state) = classic::logic::reset();

    let board = Arc::new(RwLock::new(board));
    let (tx, rx) = mpsc::channel();
    classic::logic::spawn_logic_thread(board.clone(), state, rx);

    classic::draw::window_loop(&mut window, board, tx);
}

fn start_snaek(mut window: PistonWindow) {
    let (board, state) = snaek::logic::reset();

    let board = Arc::new(RwLock::new(board));
    let (tx, rx) = mpsc::channel();
    snaek::logic::spawn_logic_thread(board.clone(), state, rx);

    snaek::draw::window_loop(&mut window, board, tx);
}
