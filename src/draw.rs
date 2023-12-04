

use std::sync::{Arc, RwLock, mpsc::Sender};

use piston_window::*;

use crate::types::{Board, B_WIDTH, B_HEIGHT, CellState};


const C_SIZE: usize = 10;

pub fn draw_board<G: Graphics>(g: &mut G, board: &Board, transform: math::Matrix2d) {
    clear(EMPTY_COLOR, g);

    for (y, row) in board.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let rect = [(x * C_SIZE) as f64, (y * C_SIZE) as f64, {(x+1) * C_SIZE} as f64, {(y+1) * C_SIZE} as f64];
            match cell {
                CellState::Empty => rectangle(EMPTY_COLOR, rect, transform, g),
                CellState::Filled => rectangle(FILLED_COLOR, rect, transform, g),
            }
        }
    }
}

pub fn create_window() -> PistonWindow {
    let window: PistonWindow = WindowSettings::new(
        "Snaek",
        [(C_SIZE * B_WIDTH) as u32, (C_SIZE * B_HEIGHT) as u32]
    )
    .build()
    .unwrap();

    window
}

pub fn window_loop(window: &mut PistonWindow, board: Arc<RwLock<Board>>, tx: Sender<Key>) {
    let mut events = Events::new(EventSettings::new().ups(10).max_fps(60));

    while let Some(event) = events.next(window) {
        // Handle update events for periodic updates
        if let Some(_upd) = event.update_args() {
            // Update logic goes here
        }

        // Handle render events for drawing
        if let Some(_r) = event.render_args() {
            let board_r = board.read().unwrap();
            window.draw_2d(&event, |context, g, _| {
                draw_board(g, &board_r, context.transform);
            });
        }

        if let Some(Button::Keyboard(key)) = event.press_args() {
            match tx.send(key) {
                Ok(_) => (),
                Err(err) => {
                    println!("Couldn't send key press {:?} because of error: {}", key, err);
                }
            }
        }
    }

}

const EMPTY_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const FILLED_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
