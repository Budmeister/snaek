use rand::Rng;

use crate::snaek::{
    types::{
        GameState,
        ShopState,
        Coord,
    },
    art::{
        PlusLava,
        BoardArt,
        PlusWater,
    }
};

use super::{
    Level,
    LevelState,
    reset_shop_rand,
    new_shop_rand
};


pub static LAKES_LEVEL: Level = Level {
    name: "Lakes",
    raw_board: LAKES_BOARD,
    index: 0,
    new_level_state: || Box::new(LakesState::new()),
};

const WEATHER_DURATION_MIN: usize = 50;
const WEATHER_DURATION_MAX: usize = 250;

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
    fn reset_shop(&mut self, s: &mut GameState) {
        reset_shop_rand(&mut s.shop, |_| 10);
    }
    fn new_shop(&mut self) -> ShopState {
        new_shop_rand(10, |_| 10)
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

static LAKES_BOARD: &[u8] = include_bytes!("../../../res/levels/lakes.bin");

