use std::{
    sync::{
        Arc,
        RwLock,
        mpsc::Sender
    },
    time::{
        Duration,
        Instant
    },
    thread,
    fs::OpenOptions,
    io::Write
};

use super::{
    logic::UserAction,
    types::{
        GameState,
        B_WIDTH,
        CellState,
        CellObject,
        CellFloor,
        SnakeColor,
        PowerupType,
        Coord,
        B_HEIGHT
    }
};

pub mod draw_sdl2;

pub use draw_sdl2::Sdl2Frontend;
use into_color::as_color;

pub trait Frontend {
    type Color: From<Color>;
    type Rect: From<Rect>;
    type ActionIterator<'a>: Iterator<Item = UserAction> + 'a where Self: 'a;

    fn new(size: (u32, u32)) -> Self;
    fn screen_size(&self) -> (u32, u32);
    fn clear(&mut self);
    fn set_color(&mut self, color: Self::Color);
    fn present(&mut self);
    fn draw_rect(&mut self, rect: Self::Rect);
    fn get_actions(&mut self) -> Self::ActionIterator<'_>;
}


///////////////////////////////////////////////////////////

pub fn window_loop<F: Frontend>(mut f: F, s: Arc<RwLock<GameState>>, tx: Sender<UserAction>) {
    // Open a file in append mode for the window loop timings
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("window_loop_times.csv")
        .unwrap();

    f.set_color(EMPTY_COLOR.into());
    f.clear();
    f.present();

    'running: loop {
        let start = Instant::now();

        for action in f.get_actions() {
            match tx.send(action) {
                Ok(_) => (),
                Err(err) => {
                    println!("Couldn't send action {:?} because of error: {}", action, err);
                }
            }
            if action == UserAction::Quit {
                break 'running;
            }
        }

        {
            let s_r = s.read().unwrap();
            draw_board(&mut f, &s_r);
        }

        f.present();

        let duration = start.elapsed();
        // Write the duration to the file
        writeln!(file, "{},", duration.as_millis()).unwrap();

        if let Some(remaining) = Duration::new(0, 1_000_000u32 / 60).checked_sub(duration) {
            thread::sleep(remaining);
        } else {
            thread::sleep(Duration::from_millis(1));
        }
    }
}

///////////////////////////////////////////////////////////

use crate::snaek::types::MAX_WATER_DIST;

/// The width of a single cell in pixels
const C_SIZE: usize = 10;

pub fn draw_board<F: Frontend>(f: &mut F, s: &GameState) {
    f.set_color(EMPTY_COLOR.into());
    f.clear();
    let (w, h) = f.screen_size();
    // In blocks
    let visible_w = w as usize / C_SIZE;
    let visible_h = h as usize / C_SIZE;
    // In pixels
    let Coord { x: snake_x, y: snake_y } = s.snake.head_pos();
    let (w, h) = (B_WIDTH as isize, B_HEIGHT as isize);

    let mut xstart = snake_x as isize - (visible_w / 2) as isize;
    let mut xstop = xstart + visible_w as isize;
    if xstop > w {
        xstart -= xstop - w;
        xstop = w;
    }
    if xstart < 0 {
        xstart = 0;
        xstop = w;
    }
    let (xstart, xstop) = (xstart as usize, xstop as usize);
    let xrange = xstart..xstop;

    let mut ystart = snake_y as isize - (visible_h / 2) as isize;
    let mut ystop = ystart + visible_h as isize;
    if ystop > h {
        ystart -= ystop - h;
        ystop = h;
    }
    if ystart < 0 {
        ystart = 0;
        ystop = h;
    }
    let (ystart, ystop) = (ystart as usize, ystop as usize);
    let yrange = ystart..ystop;

    for (y, row) in s.board[yrange].iter().enumerate() {
        for (x, cell) in row[xrange.clone()].iter().enumerate() {
            let rect = ((x * C_SIZE) as i32, (y * C_SIZE) as i32, {(x+1) * C_SIZE} as u32, {(y+1) * C_SIZE} as u32);
            let color = get_cell_color(*cell, s);
            f.set_color(color.into());
            f.draw_rect(rect.into());
        }
    }
}

fn get_cell_color(cell: CellState, s: &GameState) -> Color {
    if cell.obj == CellObject::None || 
            (cell.obj.is_powerup() && (s.frame_num / 2) % 2 == 0) ||
            (cell.obj.is_super_powerup() && s.frame_num % 2 == 0)
    {
        get_floor_color(cell.floor)
    } else {
        get_object_color(cell.obj, s)
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
        CellFloor::ExplIndicator => EXPLOSIVE_COLOR,
    }
}

fn get_object_color(obj: CellObject, s: &GameState) -> Color {
    match obj {
        CellObject::None => EMPTY_COLOR,
        CellObject::Wall => WALL_COLOR,
        CellObject::Snake(SnakeColor::DarkRed, _) => SNAKE_COLOR_DARK_RED,
        CellObject::Snake(SnakeColor::LightRed, _) => SNAKE_COLOR_LIGHT_RED,
        CellObject::Snake(SnakeColor::Head, _) => if s.invinc_time != 0 { SNAKE_COLOR_HEAD_WITH_INVINC } else { SNAKE_COLOR_HEAD },
        CellObject::Food(..) => FOOD_COLOR,
        CellObject::Powerup(pwr, ..) | CellObject::SuperPowerup(pwr, ..) => match pwr {
            PowerupType::Water => WATER_COLOR,
            PowerupType::Explosive => EXPLOSIVE_COLOR,
            PowerupType::Turf => TURF_COLOR,
            PowerupType::Seed => SEED_COLOR,
            PowerupType::Invincibility => INVINC_COLOR,
        }
        CellObject::Border => BORDER_COLOR,
    }
}

type Rect = (i32, i32, u32, u32);
type Color = (u8, u8, u8);

// Floor colors
const EMPTY_COLOR: Color = as_color!("#ffffff");
const WATER_COLOR: Color = as_color!("#3f38ff");
const LAVA_COLOR: Color = as_color!("#fcb103");
const TURF_COLOR: Color = as_color!("#94ff8c");

// Object colors
const WALL_COLOR: Color = as_color!("#000000");
const FOOD_COLOR: Color = as_color!("#11ff00");
const SEED_COLOR: Color = as_color!("#065e00");
const SEED_COLORS: [Color; MAX_WATER_DIST] = [
    as_color!("#2a5e00"),
    as_color!("#2a5904"),
    as_color!("#3a5904"),
    as_color!("#455904"),
    as_color!("#595904"),
    as_color!("#59540b"),
    as_color!("#594e04"),
    as_color!("#5e5200"),
];
const DEAD_SEED_COLOR: Color = as_color!("#542d1c");
const BORDER_COLOR: Color = as_color!("#42005e");
const SNAKE_COLOR_LIGHT_RED: Color = as_color!("#ff6038");
const SNAKE_COLOR_DARK_RED: Color = as_color!("#871d03");
const SNAKE_COLOR_HEAD: Color = as_color!("#eb9b2d");
const SNAKE_COLOR_HEAD_WITH_INVINC: Color = as_color!("#f8ffbd");

// Powerup colors
const EXPLOSIVE_COLOR: Color = as_color!("#696969");
const INVINC_COLOR: Color = as_color!("#000000");
