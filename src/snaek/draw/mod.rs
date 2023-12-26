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
        B_HEIGHT, IndicatorType, Board, LOGIC_MAX_MSPT, DRAW_MAX_USPT
    }, levels, art::BoardArt
};

use crate::sized_color_space;

pub mod draw_sdl2;

pub use draw_sdl2::Sdl2Frontend;
use into_color::{as_color, color_space};

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

    f.set_color(EMPTY_COLOR.into());
    f.clear();
    f.present();

    let mut v = reset_view_state();

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

        let lock_time;
        {
            let lock_start = start.elapsed();
            let s_r = s.read().unwrap();
            let lock_gotten = start.elapsed();
            lock_time = lock_gotten - lock_start;
            draw_board(&mut f, &s_r, &mut v);
        }

        f.present();

        let display_time = start.elapsed();
        // Write the duration to debug info
        v.debug_info.lock_uspt = lock_time.as_micros();
        v.debug_info.disp_uspt = display_time.as_micros();

        if let Some(remaining) = Duration::new(0, 1000 * DRAW_MAX_USPT as u32).checked_sub(display_time) {
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

pub fn draw_board<F: Frontend>(f: &mut F, s: &GameState, v: &mut ViewState) {
    f.set_color(EMPTY_COLOR.into());
    f.clear();
    let (w, h) = f.screen_size();

    // The size in pixels of a cell in the scoreboard
    let sb_csize = h as usize / SB_HEIGHT;
    let sb_x = w as usize - sb_csize * SB_WIDTH;

    // In blocks
    let visible_w = sb_x / C_SIZE;
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

    // Start drawing at the top and left -- this also means we have room for the scoreboard
    // Draw Board
    for (y, row) in s.board[yrange].iter().enumerate() {
        for (x, cell) in row[xrange.clone()].iter().enumerate() {
            let rect = ((x * C_SIZE) as i32, (y * C_SIZE) as i32, C_SIZE as u32, C_SIZE as u32);
            if let Some(color) = get_cell_color(*cell, s) {
                f.set_color(color.into());
                f.draw_rect(rect.into());
            }
        }
    }

    // Draw scoreboard
    for (y, row) in v.scoreboard[..].iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let rect = ((x * sb_csize + sb_x) as i32, (y * sb_csize) as i32, (sb_csize+1) as u32, (sb_csize+1) as u32);
            if let Some(color) = get_cell_color(*cell, s) {
                f.set_color(color.into());
                f.draw_rect(rect.into());
            }
        }
    }

    // Draw debug screen
    if s.debug_screen {
        ////////////////////////////////////
        // Set debug screen
        v.debug_screen.rect((0,0), (DS_WIDTH, DS_HEIGHT), CellFloor::Indicator(IndicatorType::Empty));

        const MSPT_NORMAL: CellFloor = CellFloor::Indicator(IndicatorType::MSPTNormal);
        const MSPT_OVER: CellFloor = CellFloor::Indicator(IndicatorType::MSPTOver);

        let mut lines = vec![(String::from("MSPT:"), MSPT_NORMAL)];
        // Game debug
        {
            let lock_uspt = s.debug_info.lock_uspt;
            let lock_mspt = lock_uspt / 1000;
            let lock_us_only = lock_uspt - lock_mspt * 1000;
            let proc_uspt = s.debug_info.proc_uspt;
            let proc_mspt = proc_uspt / 1000;
            let proc_us_only = proc_uspt - proc_mspt * 1000;
            let cell = if ((lock_uspt + proc_uspt) / 1000) as u64 >= LOGIC_MAX_MSPT { MSPT_OVER } else { MSPT_NORMAL };
            lines.push((format!("Ll: {}.{}", lock_mspt, lock_us_only / 100), cell));
            lines.push((format!("Lp: {}.{}", proc_mspt, proc_us_only / 100), cell));
            lines.push((format!("LM: {}.0", LOGIC_MAX_MSPT), cell));
        };

        // Draw debug
        {
            let lock_uspt = v.debug_info.lock_uspt;
            let lock_mspt = lock_uspt / 1000;
            let lock_us_only = lock_uspt - lock_mspt * 1000;
            let disp_uspt = v.debug_info.disp_uspt;
            let disp_mspt = disp_uspt / 1000;
            let disp_us_only = disp_uspt - disp_mspt * 1000;
            let max_uspt = DRAW_MAX_USPT;
            let max_mspt = max_uspt / 1000;
            let max_us_only = max_uspt - max_mspt * 1000;
            let cell = if lock_uspt + disp_uspt >= DRAW_MAX_USPT { MSPT_OVER } else { MSPT_NORMAL };
            lines.push((format!("Dl: {}.{}", lock_mspt, lock_us_only / 100), cell));
            lines.push((format!("Dd: {}.{}", disp_mspt, disp_us_only / 100), cell));
            lines.push((format!("DM: {}.{}", max_mspt, max_us_only / 100), cell));
        };

        for (i, (line, fill)) in lines.into_iter().enumerate() {
            let y = i * 6 + 1;
            let x = 1;
            v.debug_screen.text(&line, (x, y), fill, ());
        }


        
        ////////////////////////////////////
        // Draw debug screen
        let ds_csize = 3;
        // let ds_x = (visible_w * C_SIZE).saturating_sub(100 * ds_csize);
        let ds_x = sb_x;
        let ds_y = (visible_h * C_SIZE).saturating_sub(100 * ds_csize);
        for (y, row) in v.debug_screen[..].iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let rect = ((x * ds_csize + ds_x) as i32, (y * ds_csize + ds_y) as i32, (ds_csize+1) as u32, (ds_csize+1) as u32);
                if let Some(color) = get_cell_color(*cell, s) {
                    f.set_color(color.into());
                    f.draw_rect(rect.into());
                }
            }
        }
    }
}

fn get_cell_color(cell: CellState, s: &GameState) -> Option<Color> {
    if cell.obj == CellObject::None || 
            (cell.obj.is_powerup() && (s.frame_num / 2) % 2 == 0) ||
            (cell.obj.is_super_powerup() && s.frame_num % 2 == 0)
    {
        get_floor_color(cell.floor, cell.elev)
    } else {
        get_object_color(cell.obj, s)
    }
}

fn get_floor_color(floor: CellFloor, elev: u8) -> Option<Color> {
    match floor {
        CellFloor::Empty => Some(TERRAIN_COLORS[elev as usize]),
        CellFloor::Water => Some(WATER_COLOR),
        CellFloor::Lava => Some(LAVA_COLOR),
        CellFloor::Turf => Some(TURF_COLOR),
        CellFloor::Seed(dist) => Some(SEED_COLORS[dist.min(MAX_WATER_DIST - 1)]),
        CellFloor::DeadSeed => Some(DEAD_SEED_COLOR),
        CellFloor::Indicator(IndicatorType::Empty) => None,
        CellFloor::Indicator(IndicatorType::Explosion) => Some(EXPLOSIVE_COLOR),
        CellFloor::Indicator(IndicatorType::Dirt) => Some(DIRT_COLOR),
        CellFloor::Indicator(IndicatorType::MSPTNormal) => Some(MSPT_NORMAL_COLOR),
        CellFloor::Indicator(IndicatorType::MSPTOver) => Some(MSPT_OVER_COLOR),
    }
}

fn get_object_color(obj: CellObject, s: &GameState) -> Option<Color> {
    match obj {
        CellObject::None => Some(EMPTY_COLOR),
        CellObject::Wall => Some(WALL_COLOR),
        CellObject::Snake(SnakeColor::DarkRed, _) => Some(SNAKE_COLOR_DARK_RED),
        CellObject::Snake(SnakeColor::LightRed, _) => Some(SNAKE_COLOR_LIGHT_RED),
        CellObject::Snake(SnakeColor::Head, _) => Some(if s.invinc_time != 0 { SNAKE_COLOR_HEAD_WITH_INVINC } else { SNAKE_COLOR_HEAD }),
        CellObject::Food(..) => Some(FOOD_COLOR),
        CellObject::Powerup(pwr, ..) | CellObject::SuperPowerup(pwr, ..) => match pwr {
            PowerupType::Water => Some(WATER_COLOR),
            PowerupType::Explosive => Some(EXPLOSIVE_COLOR),
            PowerupType::Turf => Some(TURF_COLOR),
            PowerupType::Seed => Some(SEED_COLOR),
            PowerupType::Invincibility => Some(INVINC_COLOR),
        }
        CellObject::Border => Some(BORDER_COLOR),
    }
}

pub const SB_WIDTH: usize = 25;
pub const SB_HEIGHT: usize = 100;
pub const DS_WIDTH: usize = 100;
pub const DS_HEIGHT: usize = 100;

pub struct ViewState {
    pub scoreboard: Board<SB_WIDTH, SB_HEIGHT>,
    pub debug_screen: Board<DS_WIDTH, DS_HEIGHT>,
    pub debug_info: ViewDebugInfo,
}

#[derive(Default)]
pub struct ViewDebugInfo {
    pub lock_uspt: u128,
    pub disp_uspt: u128,
}

fn reset_view_state() -> ViewState {
    ViewState {
        scoreboard: Board::<SB_WIDTH, SB_HEIGHT>::from_bytes(levels::SCORE_BANNER_VERT),
        debug_screen: Board::<DS_WIDTH, DS_HEIGHT>::new_filled(CellFloor::Indicator(IndicatorType::Empty)),
        debug_info: ViewDebugInfo::default(),
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

// Other colors
const DIRT_COLOR: Color = as_color!("#422417");


// Terrain colors
sized_color_space!{
    TERRAIN_COLORS = [
        ("#ffffff", 0.0),
        ("#828282", 0.15),
        ("#007103", 0.4),
        ("#bab783", 0.42),
        ("#7b3c02", 0.44),
        ("#008d71", 0.55),
        ("#000000", 1.0)
    ],
    NUM_TERRAIN_COLORS = 256
}

// Display text colors
const MSPT_NORMAL_COLOR: Color = as_color!("#b7eb34");
const MSPT_OVER_COLOR: Color = as_color!("#eb4034");


// Macros for this module
mod macros {
    #[macro_export]
    macro_rules! sized_color_space {
        ($name:ident = [ $( $color:expr ),* ], $len_name:ident = $len:literal) => {
            const $name: [(u8, u8, u8); $len] = color_space!([ $( $color ),* ], $len);
            const $len_name: usize = $len;
        };
    }
}
