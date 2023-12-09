// #![windows_subsystem = "windows"]

use std::sync::{Arc, RwLock, mpsc};

use draw::create_window;

mod global;
mod draw;
mod classic;
mod snaek;
mod text;
mod sdl_example;

fn main() {
    // start_classic();
    // start_snaek_piston();
    start_snaek_sdl();
}


fn start_classic() {
    let mut window = create_window((global::W_WIDTH, global::W_HEIGHT));

    let (board, state) = classic::logic::reset();

    let board = Arc::new(RwLock::new(board));
    let (tx, rx) = mpsc::channel();
    classic::logic::spawn_logic_thread(board.clone(), state, rx);

    classic::draw::window_loop(&mut window, board, tx);
}

fn start_snaek_piston() {
    let mut window = create_window((global::W_WIDTH, global::W_HEIGHT));

    let state = snaek::logic::reset();

    let state = Arc::new(RwLock::new(state));
    let (tx, rx) = mpsc::channel();
    snaek::logic::spawn_logic_thread(state.clone(), rx);

    snaek::draw::draw_piston::window_loop(&mut window, state, tx);
}

fn start_snaek_sdl() {
    let (mut canvas, sdl_context) = snaek::draw::draw_sdl2::create_window((global::W_WIDTH, global::W_HEIGHT));

    let state = snaek::logic::reset();

    let state = Arc::new(RwLock::new(state));
    let (tx, rx) = mpsc::channel();
    snaek::logic::spawn_logic_thread(state.clone(), rx);

    snaek::draw::draw_sdl2::window_loop(&mut canvas, &sdl_context, state, tx);
}
