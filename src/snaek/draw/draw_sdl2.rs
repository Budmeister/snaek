

use sdl2::{
    render::WindowCanvas,
    Sdl,
    event::{
        Event,
        EventPollIterator
    },
    keyboard::Keycode
};

use super::{
    Frontend,
    super::logic::UserAction
};

pub struct Sdl2Frontend {
    canvas: sdl2::render::WindowCanvas,
    event_pump: sdl2::EventPump,
}
impl Frontend for Sdl2Frontend {
    type Color = (u8, u8, u8);
    type Rect = (i32, i32, u32, u32);
    type ActionIterator<'a> = std::iter::FilterMap<EventPollIterator<'a>, fn(Event) -> Option<UserAction>>;
    
    fn new(size: (u32, u32)) -> Sdl2Frontend {
        let (canvas, sdl_context) = create_window(size);
        let event_pump = sdl_context.event_pump().expect("Unable to create event pump");
        Sdl2Frontend { canvas, event_pump }
    }
    fn screen_size(&self) -> (u32, u32) {
        self.canvas.output_size().expect("Unable to find the window size")
    }
    fn clear(&mut self) {
        self.canvas.clear();
    }
    fn set_color(&mut self, color: Self::Color) {
        self.canvas.set_draw_color(color);
    }
    fn present(&mut self) {
        self.canvas.present();
    }
    fn draw_rect(&mut self, rect: Self::Rect) {
        match self.canvas.fill_rect(Some(rect.into())) {
            Ok(_) => {}
            Err(err) => println!("Error while drawing rect {}", err),
        }
    }
    fn get_actions(&mut self) -> Self::ActionIterator<'_> {
        self.event_pump
            .poll_iter()
            .filter_map(|event| {
                match event {
                    Event::Quit{..} => Some(UserAction::Quit),
                    Event::KeyDown { keycode: Some(keycode), .. } => {
                        key_to_user_action(keycode)
                    }
                    _ => None
                }
            })
    }
}

pub fn create_window((width, height): (u32, u32)) -> (WindowCanvas, Sdl) {
    let sdl_context = sdl2::init().expect("Unable to initialize sdl context");
    let video_subsystem = sdl_context.video().expect("Unable to get video subsystem");
 
    let window = video_subsystem.window("Snaek", width, height)
        .position_centered()
        .build()
        .expect("Unable to build window");
 
    let canvas = window.into_canvas().build().expect("Unable to convert window into canvas");

    (canvas, sdl_context)
}

fn key_to_user_action(keycode: Keycode) -> Option<UserAction> {
    match keycode {
        Keycode::Up => Some(UserAction::Up),
        Keycode::Left => Some(UserAction::Left),
        Keycode::Down => Some(UserAction::Down),
        Keycode::Right => Some(UserAction::Right),
        Keycode::F => Some(UserAction::Restart),
        Keycode::F3 => Some(UserAction::Debug),
        _ => None,
    }
}