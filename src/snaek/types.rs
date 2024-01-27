
use std::{ops::{Range, Deref, DerefMut, Add}, mem::MaybeUninit};

use rand::{Rng, distributions::{Distribution, Standard}};

use crate::snaek::levels::LEVELS;

use super::{levels::{Level, LevelState}, art::Fill};

#[derive(Clone, Copy, Hash, PartialEq, Default, Debug)]
pub enum CellFloor {
    #[default]
    Empty,
    Water { depth: u8 },
    Lava { depth: u8 },
    Seed { height: u8, saturation: i8 },
    Indicator(IndicatorType),
}
impl CellFloor {
    #[inline(always)]
    pub fn height(&self) -> u8 {
        match self {
            CellFloor::Empty | CellFloor::Indicator(..) => 0,
            CellFloor::Water { depth: height } | CellFloor::Lava { depth: height } | CellFloor::Seed { height, .. } => *height,
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Default, Debug)]
pub enum CellObject {
    #[default]
    None,
    Wall,
    Snake(SnakeColor, usize),
    Food(usize),
    Border,
}

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub enum PowerupType {
    Water,
    Explosive,
    Shovel,
    Seed,
    Invincibility,
}
impl Distribution<PowerupType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PowerupType {
        match rng.gen_range(0..5) {
            0 => PowerupType::Water,
            1 => PowerupType::Explosive,
            2 => PowerupType::Shovel,
            3 => PowerupType::Invincibility,
            _ => PowerupType::Seed,
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub enum IndicatorType {
    Empty,
    MSPTNormal,
    MSPTOver,
    Coin,
    PM,
    Powerup(PowerupType),
}

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub enum SnakeColor {
    DarkRed,
    LightRed,
    Head,
}

// Max seed height = fertility + saturation
pub const MAX_FERTILITY: i8 = 15;
pub const MAX_SEED_HEIGHT: i8 = 30;
pub const MAX_SATURATION: i8 = 15;
pub const MIN_SATURATION_FOR_SEED_SPREAD: i8 = 4;

#[derive(Clone, Copy, Hash, PartialEq, Default, Debug)]
pub struct CellState {
    pub floor: CellFloor,
    pub obj: CellObject,
    pub elev: u8,
    pub fertility: i8,
}
impl CellState {
    #[inline(always)]
    pub fn roof(&self) -> u8 {
        self.elev.saturating_add(self.floor.height())
    }
}

/// The width of the board in cells. Must be less than `isize::MAX`
pub const B_WIDTH: usize = 200;
/// The height of the board in cells. Must be less than `isize::MAX`
pub const B_HEIGHT: usize = 160;

type BoardArray<const W: usize, const H: usize> = [[CellState; W]; H];

#[derive(Clone, Hash, PartialEq, Debug)]
pub struct Board<const W: usize = B_WIDTH, const H: usize = B_HEIGHT>(Box<BoardArray<W, H>>);
impl<const W: usize, const H: usize> Board<W, H> {
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

    pub fn new_filled(fill: impl Fill) -> Self {
        let mut cell = CellState { floor: CellFloor::Empty, obj: CellObject::None, elev: 0, fertility: 0 };
        cell.update(fill);
        Self(Box::new([[cell; W]; H]))
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut board_vec: Vec<Vec<CellState>> = (0..H)
            .map(|_| vec![CellState::default(); W])
            .collect();

        for (i, [&floor, &elev, &fertility]) in bytes.iter().chunks::<3>().enumerate() {
            let x = i % W;
            let y = i / W;

            let fertility = fertility as i8;
            if y < H {
                board_vec[y][x] = match floor {
                    0x0 => CellState { floor: CellFloor::Empty, obj: CellObject::None, elev, fertility },
                    0x1 => CellState { floor: CellFloor::Water { depth: 1 }, obj: CellObject::None, elev, fertility },
                    0x2 => CellState { floor: CellFloor::Lava { depth: 1 }, obj: CellObject::None, elev, fertility },
                    0x3 => CellState { floor: CellFloor::Empty, obj: CellObject::Wall, elev, fertility },
                    0x4 => CellState { floor: CellFloor::Empty, obj: CellObject::Border, elev, fertility },
                    0x5 => CellState { floor: CellFloor::Seed { height: 1, saturation: MAX_SATURATION }, obj: CellObject::None, elev, fertility },
                    0x6 => CellState { floor: CellFloor::Indicator(IndicatorType::Coin), obj: CellObject::None, elev, fertility },
                    0x7 => CellState { floor: CellFloor::Indicator(IndicatorType::PM), obj: CellObject::None, elev, fertility },
                    _ => CellState::default(),
                };
            }
        }

        let board: Box<BoardArray<W, H>> = board_vec
            .into_iter()
            .map(|row| {
                let boxed_row: Box<[CellState; W]> = row.into_boxed_slice().try_into().expect("Row had incorrect length");
                *boxed_row
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
            .try_into()
            .expect("Board had incorrect height");

        Board(board)
    }
}
impl<const W: usize, const H: usize> Deref for Board<W, H> {
    type Target = BoardArray<W, H>;

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

    pub fn inner_cells_horiz_mut_enumerate<T, const W: usize>(slice: &mut [[T; W]], start_at_y: usize) -> impl Iterator<Item = (Coord, &mut T)> {
        slice
            .iter_mut()
            .enumerate()
            .flat_map(move |(y, row)| row[1..W - 1]
                .iter_mut()
                .enumerate()
                .map(move |(x, cell)| (Coord { x, y: y + start_at_y }, cell))
            )
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
    pub fn add_checked(self, rhs: Dir) -> Option<Coord> {
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
    pub fn sub_checked(self, rhs: Dir) -> Option<Coord> {
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
            y: rng.gen_range(1..(B_HEIGHT-1)),
        }
    }
}
impl Add<Dir> for Coord {
    type Output = Coord;
    /// This can be dangerous if you could have overflows
    #[inline(always)]
    fn add(self, rhs: Dir) -> Self::Output {
        let (dx, dy) = rhs.get_diff();
        let (new_x, new_y) = (self.x as isize + dx, self.y as isize + dy);

        Coord { x: new_x as usize, y: new_y as usize }
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

pub const LOGIC_MAX_MSPT: u64 = 100;
pub const DRAW_MAX_USPT: u128 = 1_000_000u128 / 60;

pub const SB_WIDTH: usize = 28;
pub const SB_HEIGHT: usize = 100;

pub struct GameState {
    pub level: &'static Level,
    pub board: Board,
    pub scoreboard: Board<SB_WIDTH, SB_HEIGHT>,
    pub shop: ShopState,

    pub snake: Snake,
    pub coins: usize,
    pub invinc_time: usize,

    pub failed: bool,
    /// The frame number from logic's perspective
    pub frame_num: usize,

    pub debug_screen: bool,
    pub debug_info: DebugInfo,

    pub salt: u32,
}
impl GameState {
    pub fn next_level(&mut self) -> Option<Box<dyn LevelState>> {
        let current_level_index = self.level.index + 1;
        if current_level_index >= LEVELS.len() {
            return None;
        }
        self.level = LEVELS[current_level_index];

        self.reset_level()
    }
    pub fn reset_level(&mut self) -> Option<Box<dyn LevelState>> {
        let l = (self.level.new_level_state)();
    
        println!("Level {}: {}", self.level.index + 1, self.level.name);
        self.board = Board::from_bytes(self.level.raw_board);
        let snake = Snake::new((5, 5), Dir::Right, self.snake.len());
        
        self.snake = snake;
        self.failed = false;

        Some(l)
    }
}

pub const NUM_SHOP_ITEMS: usize = 3;
pub struct ShopState {
    pub powerups: [ShopItem; NUM_SHOP_ITEMS],
    pub selected: usize,
    pub price_multiplier: usize,
}
impl ShopState {
    #[inline]
    pub fn get_selected(&self) -> ShopItem {
        self.powerups[self.selected]
    }
    #[inline]
    pub fn get_selected_ref(&self) -> &ShopItem {
        &self.powerups[self.selected]
    }
    #[inline]
    pub fn get_selected_mut(&mut self) -> &mut ShopItem {
        &mut self.powerups[self.selected]
    }
}
#[derive(Clone, Copy)]
pub struct ShopItem {
    pub kind: PowerupType,
    pub price: usize,
}

#[derive(Default)]
pub struct DebugInfo {
    pub lock_uspt: u128,
    pub proc_uspt: u128,
}

trait Boxed {
    fn boxed(self) -> Box<Self>
        where Self: Sized
    {
        Box::new(self)
    }
}
impl<T: Sized> Boxed for T {}

struct Chunks<I, T, const C: usize>
    where I: Iterator<Item = T>
{
    iter: I,
}
trait IntoChunks<T> {
    fn chunks<const C: usize>(self) -> Chunks<Self, T, C> where Self: Iterator<Item = T> + Sized;
}
impl<I, T> IntoChunks<T> for I
    where I: Iterator<Item = T> + Sized
{
    fn chunks<const C: usize>(self) -> Chunks<Self, T, C> where Self: Iterator<Item = T> + Sized {
        Chunks { iter: self }
    }
}
impl<I, T, const C: usize> Iterator for Chunks<I, T, C>
    where
        I: Iterator<Item = T>,
        T: Sized,
{
    type Item = [T; C];
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            // Safety: This is safe because the thing we are claiming to have initialized
            // is a bunch of MaybeUninits
            let mut item: [MaybeUninit<T>; C] = MaybeUninit::uninit().assume_init();
            for i in 0..C {
                if let Some(t) = self.iter.next() {
                    item[i].write(t);
                } else {
                    // Drop all previously iterated objects
                    for (j, to_drop) in item.into_iter().enumerate() {
                        to_drop.assume_init();
                        if j >= i {
                            break;
                        }
                    }
                    return None;
                }
            }
            // Can't transmute the array because Rust can't tell that MaybeUninit<T> and T
            // have the same size because T is generic
            let item_copy = std::mem::transmute_copy(&item);
            std::mem::forget(item);
            Some(item_copy)
        }
    }
}

pub fn proc_array<T, F: FnMut(usize) -> T, const N: usize>(mut gen: F) -> [T; N] {
    unsafe {
        // Safety: This is safe because the thing we are claiming to have initialized
        // is a bunch of MaybeUninits
        let mut item: [MaybeUninit<T>; N] = MaybeUninit::uninit().assume_init();
        for i in 0..N {
            item[i].write(gen(i));
        }
        // Can't transmute the array because Rust can't tell that MaybeUninit<T> and T
        // have the same size because T is generic
        let item_copy = std::mem::transmute_copy(&item);
        std::mem::forget(item);
        item_copy
    }
}
