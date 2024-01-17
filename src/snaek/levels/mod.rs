use rand::Rng;

use super::types::{
    GameState,
    PowerupType,
    ShopState,
    ShopItem, proc_array
};

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

pub trait LevelState: Send {
    fn update(&mut self, s: &mut GameState);
    fn reset_shop(&mut self, s: &mut GameState);
    fn new_shop(&mut self) -> ShopState;
}

pub fn new_shop_rand<F: FnMut(PowerupType) -> usize>(price_multiplier: usize, mut price: F) -> ShopState {
    ShopState {
        powerups: proc_array(|_| {
            let kind = rand::random();
            let price = price(kind);
            ShopItem { kind, price }
        }),
        selected: 0,
        price_multiplier,
    }
}

pub fn reset_shop_rand<F: FnMut(PowerupType) -> usize>(shop: &mut ShopState, mut price: F) {
    for item in &mut shop.powerups {
        let kind = rand::random();
        let price = price(kind);
        *item = ShopItem {
            kind,
            price,
        }
    }
}

// pub static _HI_LEVEL: &[u8] = include_bytes!("../../res/levels/hi.bin");
// pub static _RIVER_LEVEL: &[u8] = include_bytes!("../../res/levels/river.bin");
// pub static LONELY_WORLD_LEVEL: &[u8] = include_bytes!("../../res/levels/lonely_world.bin");
// pub static WATER_V_LAVA_LEVEL: &[u8] = include_bytes!("../../res/levels/water_v_lava.bin");
// pub static THREE_BASINS_LEVEL: &[u8] = include_bytes!("../../res/levels/three_basins.bin");
// pub static HUMBLE_BEGINNINGS_LEVEL: &[u8] = include_bytes!("../../res/levels/humble_beginnings.bin");
// pub static RIVERS_LEVEL: &[u8] = include_bytes!("../../res/levels/rivers.bin");
