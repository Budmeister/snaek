use crate::snaek::{
    types::{
        GameState,
        ShopState,
        ShopItem,
        PowerupType,
        NUM_SHOP_ITEMS,
    },
    art::{
        PlusLava,
        BoardArt,
    }
};

use super::{
    Level,
    LevelState,
    reset_shop_rand,
    new_shop_rand
};


pub static VOLCANO_LEVEL: Level = Level {
    name: "Volcano",
    raw_board: VOLCANO_BOARD,
    index: 0,
    new_level_state: || Box::new(VolcanoState::new()),
};

struct VolcanoState {
    shop_reset_count: usize,
}
impl VolcanoState {
    fn new() -> VolcanoState {
        VolcanoState {
            shop_reset_count: 0
        }
    }
}
impl LevelState for VolcanoState {
    fn update(&mut self, s: &mut GameState) {
        s.board.pt((95, 75), PlusLava(1));
    }
    fn reset_shop(&mut self, s: &mut GameState) {
        match self.shop_reset_count {
            0 => s.shop.powerups = [ShopItem { kind: PowerupType::Water, price: 10 }; NUM_SHOP_ITEMS],
            1 => s.shop.powerups = [ShopItem { kind: PowerupType::Seed, price: 10 }; NUM_SHOP_ITEMS],
            _ => reset_shop_rand(&mut s.shop, |_| 10),
        }
        self.shop_reset_count += 1;
    }
    fn new_shop(&mut self) -> ShopState {
        new_shop_rand(10, |_| 10)
    }
}

static VOLCANO_BOARD: &[u8] = include_bytes!("../../../res/levels/volcano.bin");
