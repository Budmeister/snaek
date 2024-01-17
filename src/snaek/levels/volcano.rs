use crate::snaek::{
    types::{
        GameState,
        ShopState,
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

struct VolcanoState;
impl VolcanoState {
    fn new() -> VolcanoState {
        VolcanoState
    }
}
impl LevelState for VolcanoState {
    fn update(&mut self, s: &mut GameState) {
        s.board.pt((95, 75), PlusLava(1));
    }
    fn reset_shop(&mut self, s: &mut GameState) {
        reset_shop_rand(&mut s.shop, |_| 10);
    }
    fn new_shop(&mut self) -> ShopState {
        new_shop_rand(10, |_| 10)
    }
}

static VOLCANO_BOARD: &[u8] = include_bytes!("../../../res/levels/volcano.bin");
