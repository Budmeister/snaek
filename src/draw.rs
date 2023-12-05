
use piston_window::*;

pub fn create_window(size: impl Into<Size>) -> PistonWindow {
    let window: PistonWindow = WindowSettings::new(
        "Snaek",
        size
    )
    .build()
    .unwrap();

    window
}
