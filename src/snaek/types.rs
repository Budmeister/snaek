
use std::ops::{Range, Deref, DerefMut};

use rand::{Rng, distributions::{Distribution, Standard}};

use super::levels::SCORE_BANNER;

#[derive(Clone, Copy, Hash, PartialEq, Default, Debug)]
pub enum CellFloor {
    #[default]
    Empty,
    Water,
    Lava,
    Turf,
    /// Holds distance from water
    Seed(usize),
    DeadSeed,
    ExplIndicator,
}

#[derive(Clone, Copy, Hash, PartialEq, Default, Debug)]
pub enum CellObject {
    #[default]
    None,
    Wall,
    Snake(SnakeColor, usize),
    Food(usize),
    Powerup(PowerupType, usize),
    SuperPowerup(PowerupType, usize),
    Border,
}
impl CellObject {
    pub fn is_powerup(&self) -> bool {
        matches!(self, Self::Powerup(..))
    }
    pub fn is_super_powerup(&self) -> bool {
        matches!(self, Self::SuperPowerup(..))
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub enum PowerupType {
    Water,
    Explosive,
    Turf,
    Seed,
    Invincibility,
}
impl Distribution<PowerupType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PowerupType {
        match rng.gen_range(0..5) {
            0 => PowerupType::Water,
            1 => PowerupType::Explosive,
            2 => PowerupType::Turf,
            3 => PowerupType::Invincibility,
            _ => PowerupType::Seed,
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub enum SnakeColor {
    DarkRed,
    LightRed,
    Head,
}

pub const MAX_WATER_DIST: usize = 8;
pub const FOOD_AND_POWERUP_LIFETIME: usize = 200;

#[derive(Clone, Copy, Hash, PartialEq, Default, Debug)]
pub struct CellState {
    pub floor: CellFloor,
    pub obj: CellObject,
}

/// The width of the board in cells. Must be less than `isize::MAX`
pub const B_WIDTH: usize = 200;
/// The height of the board in cells. Must be less than `isize::MAX`
pub const B_HEIGHT: usize = 160;
/// The height of the game area not including the score banner
pub const G_HEIGHT: usize = 60;

type BoardArray<const W: usize, const H: usize> = [[CellState; W]; H];

#[derive(Clone, Hash, PartialEq, Debug)]
pub struct Board<const W: usize = B_WIDTH, const H: usize = B_HEIGHT>(Box<BoardArray<W, H>>);
impl Board {
    pub fn surrounding(&self) -> impl Iterator<Item = (&CellState, [&CellState; 8])> {
        board_ops::surrounding(self.0.deref())
    }

    pub fn cell_at(&self, coord: impl Into<Coord>) -> CellState {
        board_ops::cell_at(self.0.deref(), coord)
    }

    pub fn cell_at_mut(&mut self, coord: impl Into<Coord>) -> &mut CellState {
        board_ops::cell_at_mut(self.0.deref_mut(), coord)
    }

    pub fn cells(&self) -> impl Iterator<Item = &CellState> {
        board_ops::cells( self.0.deref())
    }

    pub fn cells_mut(&mut self) -> impl Iterator<Item = &mut CellState> {
        board_ops::cells_mut(self.0.deref_mut())
    }

    pub fn inner_cells(&self) -> impl Iterator<Item = &CellState> {
        board_ops::inner_cells(self.0.deref())
    }

    pub fn inner_cells_mut(&mut self) -> impl Iterator<Item = &mut CellState> {
        board_ops::inner_cells_mut(self.0.deref_mut())
    }

    pub fn inner_cells_horiz(&self) -> impl Iterator<Item = &CellState> {
        board_ops::inner_cells_horiz(self.0.deref())
    }

    pub fn inner_cells_horiz_mut(&mut self) -> impl Iterator<Item = &mut CellState> {
        board_ops::inner_cells_horiz_mut(self.0.deref_mut())
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut board_vec: Vec<Vec<CellState>> = (0..B_HEIGHT)
            .map(|_| vec![CellState::default(); B_WIDTH])
            .collect();

        for (i, &byte) in bytes.iter().chain(SCORE_BANNER.iter()).enumerate() {
            let x = i % B_WIDTH;
            let y = i / B_WIDTH;

            if y < B_HEIGHT {
                board_vec[y][x] = match byte {
                    0x0 => CellState { floor: CellFloor::Empty, obj: CellObject::None },
                    0x1 => CellState { floor: CellFloor::Water, obj: CellObject::None },
                    0x2 => CellState { floor: CellFloor::Lava, obj: CellObject::None },
                    0x3 => CellState { floor: CellFloor::Turf, obj: CellObject::None },
                    0x4 => CellState { floor: CellFloor::Empty, obj: CellObject::Wall },
                    0x5 => CellState { floor: CellFloor::Empty, obj: CellObject::Border },
                    0x6 => CellState { floor: CellFloor::Seed(0), obj: CellObject::None },
                    0x7 => CellState { floor: CellFloor::ExplIndicator, obj: CellObject::None },
                    _ => CellState::default(),
                };
            }
        }

        let board: Box<BoardArray<B_WIDTH, B_HEIGHT>> = board_vec
            .into_iter()
            .map(|row| {
                let boxed_row: Box<[CellState; B_WIDTH]> = row.into_boxed_slice().try_into().expect("Row had incorrect length");
                *boxed_row
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
            .try_into()
            .expect("Board had incorrect height");

        Board(board)
    }
}
impl Deref for Board {
    type Target = BoardArray<B_WIDTH, B_HEIGHT>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Board {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

macro_rules! window3 {
    ($expr:expr) => {
        zip3!($expr.into_iter(), $expr.into_iter().skip(1), $expr.into_iter().skip(2))
    }
}

macro_rules! zip3 {
    ($first:expr, $second:expr, $third:expr) => {
        $first.into_iter().zip($second).zip($third).map(
            |((first, second), third)| (first, second, third)
        )
    }
}

/// Board Ops and Iterators
/// I wish I could implement these directly on the slice, but I can't :(
/// Even with extension traits, it's not possible to implement functions
/// that return iterators (impl Iterator) for foreign types.
pub mod board_ops {
    use super::Coord;

    pub fn surrounding<T, const W: usize>(slice: &[[T; W]]) -> impl Iterator<Item = (&T, [&T; 8])> {
        window3!(slice.iter())
            .map(|(top, middle, bottom)| {
                zip3!(
                    window3!(top.iter()),
                    window3!(middle.iter()),
                    window3!(bottom.iter())
                )
            })
            .flatten()
            .map(|(
                (c1, c2, c3),
                (c4, c5, c6),
                (c7, c8, c9)
            )| {
                (c5, [c1, c2, c3, c4, c6, c7, c8, c9])
            })
    }
    pub fn cell_at<T: Copy, const W: usize>(slice: &[[T; W]], coord: impl Into<Coord>) -> T {
        let Coord { x, y } = coord.into();
        slice[y][x]
    }

    pub fn cell_at_mut<T, const W: usize>(slice: &mut [[T; W]], coord: impl Into<Coord>) -> &mut T {
        let Coord { x, y } = coord.into();
        &mut slice[y][x]
    }

    pub fn cells<T, const W: usize>(slice: &[[T; W]]) -> impl Iterator<Item = &T> {
        slice
            .iter()
            .flat_map(|row| row.iter())
    }

    pub fn cells_mut<T, const W: usize>(slice: &mut [[T; W]]) -> impl Iterator<Item = &mut T> {
        slice
            .iter_mut()
            .flat_map(|row| row.iter_mut())
    }

    pub fn inner_cells<T, const W: usize>(slice: &[[T; W]]) -> impl Iterator<Item = &T> {
        slice[1..slice.len() - 1]
            .iter()
            .flat_map(|row| row[1..W - 1].iter())
    }

    pub fn inner_cells_mut<T, const W: usize>(slice: &mut [[T; W]]) -> impl Iterator<Item = &mut T> {
        let h = slice.len();
        slice[1..h - 1]
            .iter_mut()
            .flat_map(|row| row[1..W - 1].iter_mut())
    }

    pub fn inner_cells_horiz<T, const W: usize>(slice: &[[T; W]]) -> impl Iterator<Item = &T> {
        slice
            .iter()
            .flat_map(|row| row[1..W - 1].iter())
    }

    pub fn inner_cells_horiz_mut<T, const W: usize>(slice: &mut [[T; W]]) -> impl Iterator<Item = &mut T> {
        slice
            .iter_mut()
            .flat_map(|row| row[1..W - 1].iter_mut())
    }
}


#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}
impl Coord {
    pub fn rand() -> Coord {
        Coord {
            x: rand::thread_rng().gen_range(0..B_WIDTH),
            y: rand::thread_rng().gen_range(0..B_HEIGHT),
        }
    }
    pub fn rand_range(x_range: Range<usize>, y_range: Range<usize>) -> Coord {
        Coord {
            x: rand::thread_rng().gen_range(x_range),
            y: rand::thread_rng().gen_range(y_range),
        }
    }
    pub fn add(self, rhs: Dir) -> Option<Coord> {
        let (dx, dy) = rhs.get_diff();
        let (new_x, new_y) = (self.x as isize + dx, self.y as isize + dy);

        // Reject if new_x/y less than 0 or greater than board dimensions
        // Because B_WIDTH < isize::MAX, then if `new_x` is negative, then `new_x as usize > B_WIDTH`.
        if (new_x as usize) < B_WIDTH && (new_y as usize) < B_HEIGHT {
            Some(Coord { x: new_x as usize, y: new_y as usize })
        } else {
            None
        }
    }
    pub fn sub(self, rhs: Dir) -> Option<Coord> {
        let (dx, dy) = rhs.get_diff();
        let (new_x, new_y) = (self.x as isize - dx, self.y as isize - dy);

        // Reject if new_x/y less than 0 or greater than board dimensions
        // Because B_WIDTH < isize::MAX, then if `new_x` is negative, then `new_x as usize > B_WIDTH`.
        if (new_x as usize) < B_WIDTH && (new_y as usize) < B_HEIGHT {
            Some(Coord { x: new_x as usize, y: new_y as usize })
        } else {
            None
        }
    }
    pub fn add_wrapped(self, rhs: Dir) -> Coord {
        let (dx, dy) = rhs.get_diff();
        let (new_x, new_y) = (self.x as isize + dx, self.y as isize + dy);

        let new_x = if new_x < 0 { B_WIDTH  - 1 } else if new_x as usize >= B_WIDTH  { 0 } else { new_x as usize };
        let new_y = if new_y < 0 { B_HEIGHT - 1 } else if new_y as usize >= B_HEIGHT { 0 } else { new_y as usize };
        Coord { x: new_x, y: new_y }
    }
    pub fn sub_wrapped(self, rhs: Dir) -> Coord {
        let (dx, dy) = rhs.get_diff();
        let (new_x, new_y) = (self.x as isize - dx, self.y as isize - dy);

        let new_x = if new_x < 0 { B_WIDTH  - 1 } else if new_x as usize >= B_WIDTH  { 0 } else { new_x as usize };
        let new_y = if new_y < 0 { B_HEIGHT - 1 } else if new_y as usize >= B_HEIGHT { 0 } else { new_y as usize };
        Coord { x: new_x, y: new_y }
    }
    pub fn in_bounds(&self) -> bool {
        self.x < B_WIDTH && self.y < B_HEIGHT
    }
}
impl From<(usize, usize)> for Coord {
    fn from(value: (usize, usize)) -> Self {
        Coord { x: value.0, y: value.1 }
    }
}
impl Distribution<Coord> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Coord {
        Coord {
            x: rng.gen_range(1..(B_WIDTH-1)),
            y: rng.gen_range(1..(G_HEIGHT-1)),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub enum Dir {
    Up, Left, Down, Right
}
impl Dir {
    fn get_diff(&self) -> (isize, isize) {
        [(0, -1), (-1, 0), (0, 1), (1, 0)][*self as usize]
    }
    fn is_opposite(&self, other: Dir) -> bool {
        (*self as usize).abs_diff(other as usize) == 2
    }
}
impl Distribution<Dir> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dir {
        match rng.gen_range(0..4) {
            0 => Dir::Up,
            1 => Dir::Left,
            2 => Dir::Down,
            _ => Dir::Right,
        }
    }
}

#[derive(Debug)]
pub struct Snake {
    head: Coord,
    dir: Dir,
    len: usize,
}
impl Snake {
    pub fn new(head: impl Into<Coord>, dir: Dir, len: usize) -> Snake {
        Snake {
            head: head.into(), dir, len
        }
    }
    pub fn advance(&mut self) {
        self.head = self.head.add_wrapped(self.dir);
    }
    pub fn add_food(&mut self, food: usize) {
        self.len += food;
    }
    pub fn point(&mut self, dir: Dir) {
        self.dir = dir;
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn head_pos(&self) -> Coord {
        self.head
    }
}

pub struct GameState {
    pub current_level: usize,
    pub board: Board,
    pub snake: Snake,
    pub timer: usize,
    /// Time in frames until invincibility is gone
    pub invinc_time: usize,
    /// Freeze after a powerup spawns
    pub powerup_freeze: usize,
    pub failed: bool,
    /// The frame number from logic's perspective
    pub frame_num: usize,

    pub water_pwrs: usize,
    pub explo_pwrs: usize,
    pub turf_pwrs: usize,
    pub seed_pwrs: usize,

    pub empty_count: usize,
    pub water_count: usize,
    pub lava_count: usize,
    pub turf_count: usize,
    pub seed_count: usize,
}

trait Boxed {
    fn boxed(self) -> Box<Self>
        where Self: Sized
    {
        Box::new(self)
    }
}
impl<T: Sized> Boxed for T {}

trait BucketsMut<T> {
    fn buckets_mut<const N: usize>(&mut self) -> Option<[&mut [T]; N]> {
        todo!()
    }
}
