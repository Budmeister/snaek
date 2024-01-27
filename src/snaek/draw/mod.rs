use std::{
    sync::{
        Arc,
        RwLock,
        mpsc::Sender
    },
    thread,
    time::{
        Duration,
        Instant
    },
    mem::size_of,
};

use super::{
    logic::UserAction,
    types::{
        Board,
        CellFloor,
        CellObject,
        CellState,
        Coord,
        GameState,
        IndicatorType,
        PowerupType,
        SnakeColor,
        B_HEIGHT,
        B_WIDTH,
        DRAW_MAX_USPT,
        LOGIC_MAX_MSPT,
        MAX_FERTILITY,
        MAX_SEED_HEIGHT,
        SB_HEIGHT,
        SB_WIDTH,
    }, 
    art::BoardArt
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
    for (y, row) in s.scoreboard[..].iter().enumerate() {
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
    if cell.obj == CellObject::None {
        get_floor_color(cell.floor, cell.elev, cell.fertility)
    } else {
        get_object_color(cell.obj, s)
    }
}

fn get_floor_color(floor: CellFloor, elev: u8, fertility: i8) -> Option<Color> {
    let fertility = if fertility < 0 { 0 } else { fertility as usize };
    match floor {
        CellFloor::Empty => Some(TERRAIN_COLORS[elev as usize][fertility]),
        CellFloor::Water { depth } => Some(WATER_COLORS[depth as usize]),
        CellFloor::Lava { depth } => Some(LAVA_COLORS[depth as usize]),
        CellFloor::Seed { height, saturation } => Some(SEED_COLORS[elev as usize][fertility][height as usize]),
        CellFloor::Indicator(IndicatorType::Empty) => None,
        CellFloor::Indicator(IndicatorType::MSPTNormal) => Some(MSPT_NORMAL_COLOR),
        CellFloor::Indicator(IndicatorType::MSPTOver) => Some(MSPT_OVER_COLOR),
        CellFloor::Indicator(IndicatorType::Coin) => Some(COIN_COLOR),
        CellFloor::Indicator(IndicatorType::PM) => Some(PM_COLOR),
        
        CellFloor::Indicator(IndicatorType::Powerup(PowerupType::Water)) => Some(WATER_COLOR),
        CellFloor::Indicator(IndicatorType::Powerup(PowerupType::Explosive)) => Some(EXPLOSIVE_COLOR),
        CellFloor::Indicator(IndicatorType::Powerup(PowerupType::Shovel)) => Some(SHOVEL_COLOR),
        CellFloor::Indicator(IndicatorType::Powerup(PowerupType::Seed)) => Some(SEED_COLOR),
        CellFloor::Indicator(IndicatorType::Powerup(PowerupType::Invincibility)) => Some(INVINC_COLOR),
    }
}

fn get_seed_color(elev: u8, fertility: i8, height: u8, saturation: i8) -> Color {
    todo!()
}

fn get_object_color(obj: CellObject, s: &GameState) -> Option<Color> {
    match obj {
        CellObject::None => Some(EMPTY_COLOR),
        CellObject::Wall => Some(WALL_COLOR),
        CellObject::Snake(SnakeColor::DarkRed, _) => Some(SNAKE_COLOR_DARK_RED),
        CellObject::Snake(SnakeColor::LightRed, _) => Some(SNAKE_COLOR_LIGHT_RED),
        CellObject::Snake(SnakeColor::Head, _) => Some(if s.invinc_time != 0 { SNAKE_COLOR_HEAD_WITH_INVINC } else { SNAKE_COLOR_HEAD }),
        CellObject::Food(..) => Some(FOOD_COLOR),
        CellObject::Border => Some(BORDER_COLOR),
    }
}

pub const DS_WIDTH: usize = 100;
pub const DS_HEIGHT: usize = 100;

pub struct ViewState {
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

// Object colors
const WALL_COLOR: Color = as_color!("#000000");
const FOOD_COLOR: Color = as_color!("#11ff00");
const SEED_COLOR: Color = as_color!("#065e00");
const BORDER_COLOR: Color = as_color!("#42005e");
const SNAKE_COLOR_LIGHT_RED: Color = as_color!("#ff6038");
const SNAKE_COLOR_DARK_RED: Color = as_color!("#871d03");
const SNAKE_COLOR_HEAD: Color = as_color!("#eb9b2d");
const SNAKE_COLOR_HEAD_WITH_INVINC: Color = INVINC_COLOR;
// const SEED_COLORS: [[Color; 256]; MAX_WATER_DIST as usize + 1] = [
//     seed_colors::SEED_HEIGHT_COLORS_0,
//     seed_colors::SEED_HEIGHT_COLORS_1,
//     seed_colors::SEED_HEIGHT_COLORS_2,
//     seed_colors::SEED_HEIGHT_COLORS_3,
//     seed_colors::SEED_HEIGHT_COLORS_4,
//     seed_colors::SEED_HEIGHT_COLORS_5,
//     seed_colors::SEED_HEIGHT_COLORS_6,
//     seed_colors::SEED_HEIGHT_COLORS_7,
//     seed_colors::SEED_HEIGHT_COLORS_8,
//     seed_colors::SEED_HEIGHT_COLORS_9,
//     seed_colors::SEED_HEIGHT_COLORS_10,
//     seed_colors::SEED_HEIGHT_COLORS_11,
//     seed_colors::SEED_HEIGHT_COLORS_12,
//     seed_colors::SEED_HEIGHT_COLORS_13,
//     seed_colors::SEED_HEIGHT_COLORS_14,
//     seed_colors::SEED_HEIGHT_COLORS_15,
//     seed_colors::SEED_HEIGHT_COLORS_16,
//     seed_colors::SEED_HEIGHT_COLORS_17,
//     seed_colors::SEED_HEIGHT_COLORS_18,
//     seed_colors::SEED_HEIGHT_COLORS_19,
//     seed_colors::SEED_HEIGHT_COLORS_20,
// ];
mod seed_colors {
    use super::sized_color_space;

    sized_color_space!{
        SEED_HEIGHT_COLORS_0 = [
            ("#62db00", 0.0),
            ("#2a5e00", 1.0)
        ],
        NUM_HEIGHT_VALUES_0 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_1 = [
            ("#65d702", 0.0),
            ("#2c5d00", 1.0)
        ],
        NUM_HEIGHT_VALUES_1 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_2 = [
            ("#68d304", 0.0),
            ("#2f5c00", 1.0)
        ],
        NUM_HEIGHT_VALUES_2 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_3 = [
            ("#6bd007", 0.0),
            ("#315c00", 1.0)
        ],
        NUM_HEIGHT_VALUES_3 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_4 = [
            ("#6fcc09", 0.0),
            ("#345b00", 1.0)
        ],
        NUM_HEIGHT_VALUES_4 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_5 = [
            ("#72c90b", 0.0),
            ("#375b00", 1.0)
        ],
        NUM_HEIGHT_VALUES_5 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_6 = [
            ("#75c50e", 0.0),
            ("#395a00", 1.0)
        ],
        NUM_HEIGHT_VALUES_6 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_7 = [
            ("#78c210", 0.0),
            ("#3c5900", 1.0)
        ],
        NUM_HEIGHT_VALUES_7 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_8 = [
            ("#7cbe12", 0.0),
            ("#3e5900", 1.0)
        ],
        NUM_HEIGHT_VALUES_8 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_9 = [
            ("#7fbb15", 0.0),
            ("#415800", 1.0)
        ],
        NUM_HEIGHT_VALUES_9 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_10 = [
            ("#82b717", 0.0),
            ("#445800", 1.0)
        ],
        NUM_HEIGHT_VALUES_10 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_11 = [
            ("#85b319", 0.0),
            ("#465700", 1.0)
        ],
        NUM_HEIGHT_VALUES_11 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_12 = [
            ("#89b01c", 0.0),
            ("#495600", 1.0)
        ],
        NUM_HEIGHT_VALUES_12 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_13 = [
            ("#8cac1e", 0.0),
            ("#4b5600", 1.0)
        ],
        NUM_HEIGHT_VALUES_13 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_14 = [
            ("#8fa920", 0.0),
            ("#4e5500", 1.0)
        ],
        NUM_HEIGHT_VALUES_14 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_15 = [
            ("#92a523", 0.0),
            ("#515500", 1.0)
        ],
        NUM_HEIGHT_VALUES_15 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_16 = [
            ("#96a225", 0.0),
            ("#535400", 1.0)
        ],
        NUM_HEIGHT_VALUES_16 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_17 = [
            ("#999e27", 0.0),
            ("#565300", 1.0)
        ],
        NUM_HEIGHT_VALUES_17 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_18 = [
            ("#9c9b2a", 0.0),
            ("#585300", 1.0)
        ],
        NUM_HEIGHT_VALUES_18 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_19 = [
            ("#9f972c", 0.0),
            ("#5b5200", 1.0)
        ],
        NUM_HEIGHT_VALUES_19 = 256
    }
    sized_color_space!{
        SEED_HEIGHT_COLORS_20 = [
            ("#a3942f", 0.0),
            ("#5e5200", 1.0)
        ],
        NUM_HEIGHT_VALUES_20 = 256
    }
}


// Powerup colors
const EXPLOSIVE_COLOR: Color = as_color!("#696969");
const INVINC_COLOR: Color = as_color!("#262626");
const SHOVEL_COLOR: Color = as_color!("#422417");

// Other colors
const COIN_COLOR: Color = as_color!("#bdb600");
const PM_COLOR: Color = as_color!("#62fa4b");

// sized_color_space!{
//     TERRAIN_COLORS = [
//         ("#000000", 0.0),
//         ("#422417", 0.5),
//         ("#ffffff", 1.0)
//     ],
//     NUM_TERRAIN_COLORS = 256
// }

const TERRAIN_COLORS: &[[Color; (MAX_FERTILITY as usize + 1)]; (u8::MAX as usize + 1)] = {
    const TERRAIN_COLORS: &[u8; size_of::<Color>() * (MAX_FERTILITY as usize + 1) * (u8::MAX as usize + 1)] = include_bytes!("../../../res/colors/terrain_colors.bin");

    // SAFETY: Casting a pointer to an array of bytes to a pointer to a 2D array of color objects
    // Alignment: Color = (u8, u8, u8) has the same alignment as u8, so alignment is not a problem. 
    // Size: The size of a multidimensional array is the product of the size of T and all the dimensions.
    //  We take care of that by ensuring above the array has such size. 
    // As long as the colors are laid out correctly, we can do this conversion.
    unsafe {
        &*(TERRAIN_COLORS.as_ptr() as *const [[Color; (MAX_FERTILITY as usize + 1)]; (u8::MAX as usize + 1)])
    }
};

const SEED_COLORS: &[[[Color; (MAX_SEED_HEIGHT as usize + 1)]; (MAX_FERTILITY as usize + 1)]; (u8::MAX as usize + 1)] = {
    const SEED_COLORS: &[u8; size_of::<Color>() * (MAX_SEED_HEIGHT as usize + 1) * (MAX_FERTILITY as usize + 1) * (u8::MAX as usize + 1)] = include_bytes!("../../../res/colors/seed_colors.bin");
    
    // SAFETY: Casting a pointer to an array of bytes to a pointer to a 2D array of color objects
    // Alignment: Color = (u8, u8, u8) has the same alignment as u8, so alignment is not a problem. 
    // Size: The size of a multidimensional array is the product of the size of T and all the dimensions.
    //  We take care of that by ensuring above the array has such size. 
    // As long as the colors are laid out correctly, we can do this conversion.
    unsafe {
        &*(SEED_COLORS.as_ptr() as *const [[[Color; (MAX_SEED_HEIGHT as usize + 1)]; (MAX_FERTILITY as usize + 1)]; (u8::MAX as usize + 1)])
    }
};

sized_color_space!{
    WATER_COLORS = [
        ("#8fafff", 0.0),
        ("#3f38ff", 0.05),
        ("#05008a", 1.0)
    ],
    NUM_WATER_COLORS = 256
}

sized_color_space!{
    LAVA_COLORS = [
        ("#ffb054", 0.0),
        ("#b5680e", 0.05),
        ("#995200", 0.7),
        ("#824805", 1.0)
    ],
    NUM_LAVA_COLORS = 256
}

// Display text colors
const MSPT_NORMAL_COLOR: Color = as_color!("#b7eb34");
const MSPT_OVER_COLOR: Color = as_color!("#eb4034");


// Macros for this module
mod macros {
    #[macro_export]
    macro_rules! sized_color_space {
        ($name:ident = [ $( $color:expr ),* ], $len_name:ident = $len:literal) => {
            pub const $name: [(u8, u8, u8); $len] = $crate::snaek::draw::color_space!([ $( $color ),* ], $len);
            pub const $len_name: usize = $len;
        };
    }
}
