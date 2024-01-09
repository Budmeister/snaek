use super::types::GameState;

pub struct Level {
    pub name: &'static str,
    pub raw_board: &'static [u8],
    pub index: usize,
    pub starting_season: fn (&mut GameState),
}

pub static LEVELS: &[Level] = &[
    Level {
        name: "Rivers",
        raw_board: RIVERS_LEVEL,
        index: 0,
        starting_season: |s| {
            
        },
    }
];

pub static _HI_LEVEL: &[u8] = include_bytes!("../../res/levels/hi.bin");
pub static _RIVER_LEVEL: &[u8] = include_bytes!("../../res/levels/river.bin");
pub static LONELY_WORLD_LEVEL: &[u8] = include_bytes!("../../res/levels/lonely_world.bin");
pub static WATER_V_LAVA_LEVEL: &[u8] = include_bytes!("../../res/levels/water_v_lava.bin");
pub static THREE_BASINS_LEVEL: &[u8] = include_bytes!("../../res/levels/three_basins.bin");
pub static HUMBLE_BEGINNINGS_LEVEL: &[u8] = include_bytes!("../../res/levels/humble_beginnings.bin");
pub static RIVERS_LEVEL: &[u8] = include_bytes!("../../res/levels/rivers.bin");

pub static SCORE_BANNER: &[u8] = include_bytes!("../../res/levels/score_banner.bin");
pub static SCORE_BANNER_VERT: &[u8] = include_bytes!("../../res/levels/score_banner_vertical.bin");
