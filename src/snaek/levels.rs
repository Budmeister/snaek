use rand::Rng;

use super::{types::{GameState, Coord, PowerupType}, art::{PlusLava, BoardArt, PlusWater}};

pub struct Level {
    pub name: &'static str,
    pub raw_board: &'static [u8],
    pub index: usize,
    pub new_level_state: fn () -> Box<dyn LevelState>,
}

pub static LEVELS: &[Level] = &[
    Level {
        name: "Lakes",
        raw_board: LAKES_LEVEL,
        index: 0,
        new_level_state: || Box::new(LakesState::new()),
    }
];

const WEATHER_DURATION_MIN: usize = 50;
const WEATHER_DURATION_MAX: usize = 250;
pub trait LevelState {
    fn update(&mut self, s: &mut GameState);
    fn choose_powerup_type(&mut self, s: &mut GameState) -> PowerupType;
}

struct LakesState {
    weather: Weather,
}
struct Weather {
    duration: usize,
    kind: WeatherType
}
enum WeatherType {
    None,
    LavaRain,
    WaterRain,
}
impl LevelState for LakesState {
    fn update(&mut self, s: &mut GameState) {
        match self.weather.kind {
            WeatherType::None => {}
            WeatherType::LavaRain => {
                let coord: Coord = rand::thread_rng().gen();
                s.board.pt(coord, PlusLava(1));
            }
            WeatherType::WaterRain => {
                let coord: Coord = rand::thread_rng().gen();
                s.board.pt(coord, PlusWater(1));
            }
        }
        if self.weather.duration > 0 {
            self.weather.duration -= 1;
        } else {
            self.next_weather();
        }
    }
    fn choose_powerup_type(&mut self, s: &mut GameState) -> PowerupType {
        rand::thread_rng().gen()
    }
}
impl LakesState {
    fn new() -> LakesState {
        Self {
            weather: Weather {
                duration: Self::rand_weather_duration(),
                kind: WeatherType::None,
            }
        }
    }
    fn next_weather(&mut self) {
        let kind = match rand::thread_rng().gen_range(0..3) {
            0 => WeatherType::None,
            1 => WeatherType::LavaRain,
            _ => WeatherType::WaterRain,
        };
        self.weather = Weather {
            duration: Self::rand_weather_duration(),
            kind,
        }
    }
    fn rand_weather_duration() -> usize {
        let duration = rand::thread_rng().gen_range(WEATHER_DURATION_MIN..WEATHER_DURATION_MAX);
        println!("Random weather duration: {}", duration);
        duration
    }
}

pub static _HI_LEVEL: &[u8] = include_bytes!("../../res/levels/hi.bin");
pub static _RIVER_LEVEL: &[u8] = include_bytes!("../../res/levels/river.bin");
pub static LONELY_WORLD_LEVEL: &[u8] = include_bytes!("../../res/levels/lonely_world.bin");
pub static WATER_V_LAVA_LEVEL: &[u8] = include_bytes!("../../res/levels/water_v_lava.bin");
pub static THREE_BASINS_LEVEL: &[u8] = include_bytes!("../../res/levels/three_basins.bin");
pub static HUMBLE_BEGINNINGS_LEVEL: &[u8] = include_bytes!("../../res/levels/humble_beginnings.bin");
pub static RIVERS_LEVEL: &[u8] = include_bytes!("../../res/levels/rivers.bin");
pub static LAKES_LEVEL: &[u8] = include_bytes!("../../res/levels/lakes.bin");

pub static SCORE_BANNER: &[u8] = include_bytes!("../../res/levels/score_banner.bin");
pub static SCORE_BANNER_VERT: &[u8] = include_bytes!("../../res/levels/score_banner_vertical.bin");
