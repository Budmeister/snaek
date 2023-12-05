
use std::{collections::VecDeque, ops::Range};

use rand::Rng;

#[derive(Clone, Copy, Hash, PartialEq, Default, Debug)]
pub enum CellState {
    #[default]
    Empty,
    Filled,
}

/// The width of the board in cells. Must be less than `isize::MAX`
pub const B_WIDTH: usize = 100;
/// The height of the board in cells. Must be less than `isize::MAX`
pub const B_HEIGHT: usize = 60;

pub type Board = [[CellState; B_WIDTH]; B_HEIGHT];
pub trait BoardExt {
    fn new() -> Self;
    fn cell_at(&self, coord: impl Into<Coord>) -> CellState;
    fn cell_at_mut(&mut self, coord: impl Into<Coord>) -> &mut CellState;
    /// This function does nothing if the given coord is out of bounds
    fn set_cell_at(&mut self, coord: impl Into<Coord>, cell: CellState) {
        let coord = coord.into();
        if coord.x >= B_WIDTH || coord.y >= B_HEIGHT {
            return;
        }
        *self.cell_at_mut(coord) = cell;
    }
}
impl BoardExt for Board {
    fn new() -> Self {
        [[CellState::default(); B_WIDTH]; B_HEIGHT]
    }
    fn cell_at(&self, coord: impl Into<Coord>) -> CellState {
        let Coord { x, y } = coord.into();
        self[y][x]
    }
    fn cell_at_mut(&mut self, coord: impl Into<Coord>) -> &mut CellState {
        let Coord { x, y } = coord.into();
        &mut self[y][x]
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
}
impl From<(usize, usize)> for Coord {
    fn from(value: (usize, usize)) -> Self {
        Coord { x: value.0, y: value.1 }
    }
}

// front is head, back is tail
pub type Joints = VecDeque<(Coord, Dir)>;

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

#[derive(Debug)]
pub struct Snake {
    joints: Joints,
    food_to_add: usize,
}
impl Snake {
    pub fn new<C: Into<Coord>>(head: C, dir: Dir, len: usize) -> Snake {
        let head = head.into();
        let mut tail = head;
        for _ in 0..len {
            tail = tail.sub_wrapped(dir);
        }
        let mut joints = VecDeque::new();
        joints.push_back((head, dir));
        joints.push_back((tail, dir));
        Snake {
            joints,
            food_to_add: 0,
        }
    }

    pub fn point(&mut self, dir: Dir) {
        let front_dir = &mut self.joints.front_mut().unwrap().1;
        if !front_dir.is_opposite(dir) {
            *front_dir = dir;
            self.joints.push_front((self.head_pos(), dir));
        }
    }

    /// Returns true if we ran into a wall
    pub fn advance(&mut self, wrap: bool) -> bool {
        if self.joints.is_empty() {
            panic!("Cannot advance a snake if it is empty");
        }

        let head = self.joints.front_mut().unwrap();
        if wrap {
            head.0 = head.0.add_wrapped(head.1);
        } else {
            let new_head = head.0.add(head.1);
            if let Some(new_head) = new_head {
                head.0 = new_head;
            } else {
                return true;
            }
        }

        // Advance the tail unless we ate some food
        if self.food_to_add == 0 {
            let tail = self.joints.back_mut().unwrap();
            tail.0 = tail.0.add_wrapped(tail.1);
            let tail_pos = self.tail_pos();
            let second_last = {
                let mut iter = self.joints.iter().rev();
                iter.next();
                iter.next().unwrap() // joints always has at least two items: a head and a tail
            };
            if tail_pos == second_last.0 {
                self.joints.pop_back();
            }
        } else {
            self.food_to_add -= 1;
        }

        false
    }

    pub fn add_food(&mut self, food: usize) {
        self.food_to_add += food;
    }

    pub fn head_pos(&self) -> Coord {
        self.joints.front().unwrap().0
    }

    pub fn tail_pos(&self) -> Coord {
        self.joints.back().unwrap().0
    }

    pub fn joints(&self) -> &Joints {
        &self.joints
    }
}

pub struct GameState {
    pub snake: Snake,
    pub snake_len: usize,
    pub food: Coord,
    pub powerup: Option<Coord>,
    pub timer: usize,
    /// Time until powerup is gone
    pub powerup_strength: usize,
    /// Freeze after a powerup spawns
    pub powerup_freeze: usize,
    pub failed: bool,
}
