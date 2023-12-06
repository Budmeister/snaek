
pub const NUM_LEVELS: usize = 3;

pub static LEVELS: [&[u8]; NUM_LEVELS] = [
    LONELY_WORLD,
    WATER_V_LAVA_LEVEL,
    THREE_BASINS_LEVEL,
];

pub static LEVEL_NAMES: [&str; NUM_LEVELS] = [
    "Lonely World",
    "Water Vs. Lava",
    "Three Basins",
];

pub static _HI_LEVEL: &[u8] = include_bytes!("../../res/levels/hi.bin");
pub static _RIVER_LEVEL: &[u8] = include_bytes!("../../res/levels/river.bin");
pub static LONELY_WORLD: &[u8] = include_bytes!("../../res/levels/lonely_world.bin");
pub static WATER_V_LAVA_LEVEL: &[u8] = include_bytes!("../../res/levels/water_v_lava.bin");
pub static THREE_BASINS_LEVEL: &[u8] = include_bytes!("../../res/levels/three_basins.bin");
