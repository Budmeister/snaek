use rand::Rng;

use super::{types::{GameState, Coord, PowerupType}, art::{PlusLava, BoardArt, PlusWater}};

mod lakes;
use lakes::LAKES_LEVEL;

mod volcano;
use volcano::VOLCANO_LEVEL;

pub static LEVELS: &[&Level] = &[
    // &LAKES_LEVEL,
    &VOLCANO_LEVEL,
];

pub struct Level {
    pub name: &'static str,
    pub raw_board: &'static [u8],
    pub index: usize,
    pub new_level_state: fn () -> Box<dyn LevelState>,
}

pub trait LevelState {
    fn update(&mut self, s: &mut GameState);
    fn choose_powerup_type(&mut self, s: &mut GameState) -> PowerupType;
}

// pub static _HI_LEVEL: &[u8] = include_bytes!("../../res/levels/hi.bin");
// pub static _RIVER_LEVEL: &[u8] = include_bytes!("../../res/levels/river.bin");
// pub static LONELY_WORLD_LEVEL: &[u8] = include_bytes!("../../res/levels/lonely_world.bin");
// pub static WATER_V_LAVA_LEVEL: &[u8] = include_bytes!("../../res/levels/water_v_lava.bin");
// pub static THREE_BASINS_LEVEL: &[u8] = include_bytes!("../../res/levels/three_basins.bin");
// pub static HUMBLE_BEGINNINGS_LEVEL: &[u8] = include_bytes!("../../res/levels/humble_beginnings.bin");
// pub static RIVERS_LEVEL: &[u8] = include_bytes!("../../res/levels/rivers.bin");

pub static SCORE_BANNER: &[u8] = include_bytes!("../../../res/levels/score_banner.bin");
pub static SCORE_BANNER_VERT: &[u8] = include_bytes!("../../../res/levels/score_banner_vertical.bin");
