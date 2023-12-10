
pub const NUM_LEVELS: usize = 4;

pub static LEVELS: [&[u8]; NUM_LEVELS] = [
    HUMBLE_BEGINNINGS_LEVEL,
    LONELY_WORLD_LEVEL,
    WATER_V_LAVA_LEVEL,
    THREE_BASINS_LEVEL,
];

pub static LEVEL_NAMES: [&str; NUM_LEVELS] = [
    "Humble Beginnings",
    "Lonely World",
    "Water Vs. Lava",
    "Three Basins",
];

pub static _HI_LEVEL: &[u8] = include_bytes!("../../res/levels/hi.bin");
pub static _RIVER_LEVEL: &[u8] = include_bytes!("../../res/levels/river.bin");
pub static LONELY_WORLD_LEVEL: &[u8] = include_bytes!("../../res/levels/lonely_world.bin");
pub static WATER_V_LAVA_LEVEL: &[u8] = include_bytes!("../../res/levels/water_v_lava.bin");
pub static THREE_BASINS_LEVEL: &[u8] = include_bytes!("../../res/levels/three_basins.bin");
pub static HUMBLE_BEGINNINGS_LEVEL: &[u8] = include_bytes!("../../res/levels/humble_beginnings.bin");
