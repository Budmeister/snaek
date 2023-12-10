
use std::{sync::{Arc, RwLock, mpsc::Sender}, time::Duration};

use sdl2::{render::WindowCanvas, Sdl, event::Event, keyboard::Keycode};

use super::super::{
    types::{
        B_WIDTH,
        CellState,
        CellFloor,
        CellObject,
        PowerupType,
        GameState
    },
    logic::UserAction
};

use crate::{global::W_WIDTH, snaek::types::MAX_WATER_DIST};

/// The width of a single cell in pixels
const C_SIZE: usize = W_WIDTH as usize / B_WIDTH;

pub fn draw_board(canvas: &mut WindowCanvas, s: &GameState) {
    canvas.set_draw_color(EMPTY_COLOR);
    canvas.clear();
    for (y, row) in s.board.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let rect = ((x * C_SIZE) as i32, (y * C_SIZE) as i32, {(x+1) * C_SIZE} as u32, {(y+1) * C_SIZE} as u32);
            let color = get_cell_color(*cell, s);
            canvas.set_draw_color(color);
            match canvas.fill_rect(Some(rect.into())) {
                Ok(_) => {},
                Err(err) => println!("Error while displaying rect: {}", err),
            };
        }
    }
}

fn get_cell_color(cell: CellState, s: &GameState) -> Color {
    if cell.obj == CellObject::None || (cell.obj.is_powerup() && s.frame_num % 2 == 0) {
        get_floor_color(cell.floor)
    } else {
        get_object_color(cell.obj)
    }
}

fn get_floor_color(floor: CellFloor) -> Color {
    match floor {
        CellFloor::Empty => EMPTY_COLOR,
        CellFloor::Water => WATER_COLOR,
        CellFloor::Lava => LAVA_COLOR,
        CellFloor::Turf => TURF_COLOR,
        CellFloor::Seed(dist) => SEED_COLORS[dist.min(MAX_WATER_DIST - 1)],
        CellFloor::DeadSeed => DEAD_SEED_COLOR,
    }
}

fn get_object_color(obj: CellObject) -> Color {
    match obj {
        CellObject::None => EMPTY_COLOR,
        CellObject::Wall => WALL_COLOR,
        CellObject::Snake(true) => SNAKE_COLOR_1,
        CellObject::Snake(false) => SNAKE_COLOR_2,
        CellObject::Food => FOOD_COLOR,
        CellObject::Powerup(pwr) => match pwr {
            PowerupType::Water => WATER_COLOR,
            PowerupType::Explosive => EXPLOSIVE_COLOR,
            PowerupType::Turf => TURF_COLOR,
            PowerupType::Seed => SEED_COLOR,
            PowerupType::Invincibility => INVINC_COLOR,
        }
        CellObject::Border => BORDER_COLOR,
    }
}

pub fn create_window((width, height): (u32, u32)) -> (WindowCanvas, Sdl) {
    let sdl_context = sdl2::init().expect("Unable to initialize sdl context");
    let video_subsystem = sdl_context.video().expect("Unable to get video subsystem");
 
    let window = video_subsystem.window("Snaek", width, height)
        .position_centered()
        .build()
        .expect("Unable to build window");
 
    let canvas = window.into_canvas().build().expect("Unable to convert window into canvas");

    (canvas, sdl_context)
}

pub fn window_loop(canvas: &mut WindowCanvas, sdl_context: &Sdl, s: Arc<RwLock<GameState>>, tx: Sender<UserAction>) {
    canvas.set_draw_color(EMPTY_COLOR);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().expect("Unable to create event pump");
    'running: loop {
        for event in event_pump.poll_iter() {

            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    if let Some(action) = key_to_user_action(keycode) {
                        match tx.send(action) {
                            Ok(_) => (),
                            Err(err) => {
                                println!("Couldn't send key press {:?} because of error: {}", keycode, err);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        {
            let s_r = s.read().unwrap();
            draw_board(canvas, &s_r);
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn key_to_user_action(keycode: Keycode) -> Option<UserAction> {
    match keycode {
        Keycode::Up => Some(UserAction::Up),
        Keycode::Left => Some(UserAction::Left),
        Keycode::Down => Some(UserAction::Down),
        Keycode::Right => Some(UserAction::Right),
        Keycode::W => Some(UserAction::Water),
        Keycode::E => Some(UserAction::Explosion),
        Keycode::R => Some(UserAction::Turf),
        Keycode::D => Some(UserAction::Seed),
        Keycode::F => Some(UserAction::Restart),
        Keycode::S => Some(UserAction::Shop),
        _ => None,
    }
}

type Color = (u8, u8, u8);

// Floor colors
const EMPTY_COLOR: Color = (0xff, 0xff, 0xff);
const WATER_COLOR: Color = (0x3f, 0x38, 0xff);
const LAVA_COLOR: Color = (0xfc, 0xb1, 0x03);
const TURF_COLOR: Color = (0x94, 0xff, 0x8c);

// Object colors
const WALL_COLOR: Color = (0x00, 0x00, 0x00);
const FOOD_COLOR: Color = (0x11, 0xff, 0x00);
const SEED_COLOR: Color = (0x06, 0x5e, 0x00); // #065e00
const SEED_COLORS: [Color; MAX_WATER_DIST] = [
    (0x2a, 0x5e, 0x00), // #2a5e00
    (0x2a, 0x59, 0x04), // #2a5904
    (0x3a, 0x59, 0x04), // #3a5904
    (0x45, 0x59, 0x04), // #455904
    (0x59, 0x59, 0x04), // #595904
    (0x59, 0x54, 0x0b), // #59540b
    (0x59, 0x4e, 0x04), // #594e04
    (0x5e, 0x52, 0x00), // #5e5200
];
const DEAD_SEED_COLOR: Color = (0x54, 0x2d, 0x1c); // #542d1c
const BORDER_COLOR: Color = (0x42, 0x00, 0x5e);
const SNAKE_COLOR_1: Color = (0xff, 0x60, 0x38);
const SNAKE_COLOR_2: Color = (0x87, 0x1d, 0x03);

// Powerup colors
const EXPLOSIVE_COLOR: Color = (0x69, 0x69, 0x69);
const INVINC_COLOR: Color = (0x00, 0x00, 0x00);
