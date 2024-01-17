// #![windows_subsystem = "windows"]

use std::sync::{Arc, RwLock, mpsc};

use draw::create_window;
use snaek::draw::Frontend;

mod global;
mod draw;
mod classic;
mod snaek;
mod text;

fn main() {
    // start_classic();
    // start_snaek_piston();
    start_snaek::<snaek::draw::Sdl2Frontend>();
}


fn start_classic() {
    let mut window = create_window((global::W_WIDTH, global::W_HEIGHT));

    let (board, state) = classic::logic::reset();

    let board = Arc::new(RwLock::new(board));
    let (tx, rx) = mpsc::channel();
    classic::logic::spawn_logic_thread(board.clone(), state, rx);

    classic::draw::window_loop(&mut window, board, tx);
}

fn start_snaek<F: Frontend>() {
    let f = F::new((global::W_WIDTH, 800));

    let (s, l) = snaek::logic::reset();

    let s = Arc::new(RwLock::new(s));
    let (tx, rx) = mpsc::channel();
    snaek::logic::spawn_logic_thread(s.clone(), l, rx);

    snaek::draw::window_loop(f, s, tx);
}
