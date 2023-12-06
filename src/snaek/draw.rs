
use std::sync::{Arc, RwLock, mpsc::Sender};

use into_color::as_color;
use piston_window::{*, types::Color};

use super::types::{B_WIDTH, CellState, CellFloor, CellObject, PowerupType, GameState};

use crate::global::W_WIDTH;

/// The width of a single cell in pixels
const C_SIZE: usize = W_WIDTH as usize / B_WIDTH;

pub fn draw_board<G: Graphics>(g: &mut G, s: &GameState, transform: math::Matrix2d) {
    clear(EMPTY_COLOR, g);
    for (y, row) in s.board.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let rect = [(x * C_SIZE) as f64, (y * C_SIZE) as f64, {(x+1) * C_SIZE} as f64, {(y+1) * C_SIZE} as f64];
            let color = get_cell_color(*cell, s);
            rectangle(color, rect, transform, g);
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
    }
}

fn get_object_color(obj: CellObject) -> Color {
    match obj {
        CellObject::None => EMPTY_COLOR,
        CellObject::Wall => WALL_COLOR,
        CellObject::Snake(true) => SNAKE_COLOR_1,
        CellObject::Snake(false) => SNAKE_COLOR_2,
        CellObject::Food => FOOD_COLOR,
        CellObject::Seed => SEED_COLOR,
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

pub fn window_loop(window: &mut PistonWindow, s: Arc<RwLock<GameState>>, tx: Sender<Key>) {
    let mut events = Events::new(EventSettings::new().ups(10).max_fps(60));

    while let Some(event) = events.next(window) {
        // Handle update events for periodic updates
        if let Some(_upd) = event.update_args() {
            // Update logic goes here
        }

        // Handle render events for drawing
        if let Some(_r) = event.render_args() {
            let s_r = s.read().unwrap();
            window.draw_2d(&event, |context, g, _| {
                draw_board(g, &s_r, context.transform);
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

// Floor colors
const EMPTY_COLOR: Color = as_color!("#ffffff");
const WATER_COLOR: Color = as_color!("#3f38ff");
const LAVA_COLOR: Color = as_color!("#fcb103");
const TURF_COLOR: Color = as_color!("#94ff8c");

// Object colors
const WALL_COLOR: Color = as_color!("#000000");
const FOOD_COLOR: Color = as_color!("#11ff00");
const SEED_COLOR: Color = as_color!("#065e00");
const BORDER_COLOR: Color = as_color!("#42005e");
const SNAKE_COLOR_1: Color = as_color!("#ff6038");
const SNAKE_COLOR_2: Color = as_color!("#871d03");

// Powerup colors
const EXPLOSIVE_COLOR: Color = as_color!("#696969");
const INVINC_COLOR: Color = as_color!("#000000");

trait TryIntoColor {
    type Error;

    fn try_into_color(self) -> Result<Color, Self::Error>;
}
impl TryIntoColor for &str {
    type Error = &'static str;

    fn try_into_color(self) -> Result<Color, Self::Error> {
        if !self.is_ascii() {
            return Err("Must be ascii string");
        }
        match self.len() {
            7 => {},
            9 => {},
            _ => return Err("Invalid string length"),
        }
        let (hash, rest) = self.split_at(1);
        if hash != "#" {
            return Err("Color string must start with '#'");
        }

        let (r, rest) = rest.split_at(2);
        let r = match u8::from_str_radix(r, 16) {
            Ok(r) => r,
            Err(_) => return Err("Unable to parse color string"),
        };

        let (g, rest) = rest.split_at(2);
        let g = match u8::from_str_radix(g, 16) {
            Ok(g) => g,
            Err(_) => return Err("Unable to parse color string"),
        };

        let (b, rest) = rest.split_at(2);
        let b = match u8::from_str_radix(b, 16) {
            Ok(b) => b,
            Err(_) => return Err("Unable to parse color string"),
        };

        let a = if !rest.is_empty() {
            let (a_, rest) = rest.split_at(2);
            match u8::from_str_radix(a_, 16) {
                Ok(a) => a,
                Err(_) => return Err("Unable to parse color string"),
            }
        } else {
            u8::MAX
        };

        Ok([
            r as f32 / u8::MAX as f32,
            g as f32 / u8::MAX as f32,
            b as f32 / u8::MAX as f32,
            a as f32 / u8::MAX as f32,
        ])
    }
}
