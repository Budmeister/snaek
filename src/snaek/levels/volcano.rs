use rand::Rng;

use crate::snaek::art::{PlusLava, BoardArt};

use super::{Level, LevelState};


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
    fn update(&mut self, s: &mut crate::snaek::types::GameState) {
        s.board.pt((95, 75), PlusLava(1));
    }
    fn choose_powerup_type(&mut self, s: &mut crate::snaek::types::GameState) -> crate::snaek::types::PowerupType {
        rand::thread_rng().gen()
    }
}

static VOLCANO_BOARD: &[u8] = include_bytes!("../../../res/levels/volcano.bin");
