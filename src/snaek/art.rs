use super::types::{Coord, Board, CellState, CellFloor, CellObject, IndicatorType, PowerupType};
use std::mem::discriminant as variant;

use crate::text::{GRIDS, C_WIDTH};

const EXPLOSION_WALK_COUNT: usize = 30;
const EXPLOSION_WALK_MAX_DIST: usize = 30;

pub trait Fill: Copy {
    fn fill(&self, cell: &mut CellState);
}
impl Fill for CellFloor {
    fn fill(&self, cell: &mut CellState) {
        cell.floor = *self;
    }
}
impl Fill for CellObject {
    fn fill(&self, cell: &mut CellState) {
        cell.obj = *self;
    }
}
impl Fill for IndicatorType {
    fn fill(&self, cell: &mut CellState) {
        CellFloor::Indicator(*self).fill(cell);
    }
}
impl Fill for PowerupType {
    fn fill(&self, cell: &mut CellState) {
        CellFloor::Indicator(IndicatorType::Powerup(*self)).fill(cell);
    }
}
impl Fill for u8 {
    fn fill(&self, cell: &mut CellState) {
        cell.elev = *self;
    }
}
impl Fill for CellState {
    fn fill(&self, cell: &mut CellState) {
        *cell = *self;
    }
}
impl Fill for () {
    fn fill(&self, cell: &mut CellState) {
        // Does nothing
    }
}
impl<F1: Fill, F2: Fill> Fill for (F1, F2) {
    fn fill(&self, cell: &mut CellState) {
        self.0.fill(cell);
        self.1.fill(cell);
    }
}
impl Fill for fn(&mut CellState) {
    fn fill(&self, cell: &mut CellState) {
        self(cell);
    }
}

#[derive(Clone, Copy)]
pub struct PlusWater(pub u8);
impl Fill for PlusWater {
    fn fill(&self, cell: &mut CellState) {
        match &mut cell.floor {
            CellFloor::Water { depth } => *depth = depth.saturating_add(self.0),
            CellFloor::Lava { depth } => {
                match self.0.cmp(depth) {
                    std::cmp::Ordering::Less => {
                        *depth -= self.0;
                    },
                    std::cmp::Ordering::Equal => {
                        cell.floor = CellFloor::Empty;
                    },
                    std::cmp::Ordering::Greater => {
                        cell.elev = cell.elev.saturating_add(*depth);
                        *depth = 0;
                    },
                }
            }
            CellFloor::Empty => cell.floor = CellFloor::Water { depth: self.0 },
            _ => {}
        }
    }
}
impl CellMatch for PlusWater {
    fn matches(&self, other: &CellState) -> bool {
        self.vmatches(other)
    }
    fn vmatches(&self, other: &CellState) -> bool {
        matches!(other.floor, CellFloor::Water { .. })
    }
}
#[derive(Clone, Copy)]
pub struct PlusLava(pub u8);
impl Fill for PlusLava {
    fn fill(&self, cell: &mut CellState) {
        match &mut cell.floor {
            CellFloor::Lava { depth } => *depth = depth.saturating_add(self.0),
            CellFloor::Water { depth } => {
                match self.0.cmp(depth) {
                    std::cmp::Ordering::Less => {
                        *depth -= self.0;
                    },
                    std::cmp::Ordering::Equal => {
                        cell.floor = CellFloor::Empty;
                    },
                    std::cmp::Ordering::Greater => {
                        cell.elev = cell.elev.saturating_add(*depth);
                        *depth = 0;
                    },
                }
            }
            CellFloor::Empty => cell.floor = CellFloor::Lava { depth: self.0 },
            _ => {}
        }
    }
}
impl CellMatch for PlusLava {
    fn matches(&self, other: &CellState) -> bool {
        self.vmatches(other)
    }
    fn vmatches(&self, other: &CellState) -> bool {
        matches!(other.floor, CellFloor::Lava { .. })
    }
}
#[derive(Clone, Copy)]
pub struct PlusSeed(pub u8);
impl Fill for PlusSeed {
    fn fill(&self, cell: &mut CellState) {
        match &mut cell.floor {
            CellFloor::Lava { depth: height } | CellFloor::Water { depth: height }  | CellFloor::Seed { height, .. } => *height = height.saturating_add(self.0),
            CellFloor::Empty => cell.floor = CellFloor::Seed { height: self.0, dist: 0 },
            _ => {}
        }
    }
}
impl CellMatch for PlusSeed {
    fn matches(&self, other: &CellState) -> bool {
        self.vmatches(other)
    }
    fn vmatches(&self, other: &CellState) -> bool {
        matches!(other.floor, CellFloor::Seed { .. })
    }
}

pub trait CellMatch {
    fn matches(&self, other: &CellState) -> bool;
    fn vmatches(&self, other: &CellState) -> bool;
}
impl CellMatch for CellFloor {
    fn matches(&self, other: &CellState) -> bool {
        *self == other.floor
    }
    fn vmatches(&self, other: &CellState) -> bool {
        variant(self) == variant(&other.floor)
    }
}
impl CellMatch for CellObject {
    fn matches(&self, other: &CellState) -> bool {
        *self == other.obj
    }
    fn vmatches(&self, other: &CellState) -> bool {
        variant(self) == variant(&other.obj)
    }
}
impl CellMatch for u8 {
    fn matches(&self, other: &CellState) -> bool {
        *self == other.elev
    }
    fn vmatches(&self, _: &CellState) -> bool {
        true
    }
}
impl<M1: CellMatch, M2: CellMatch> CellMatch for (M1, M2) {
    fn matches(&self, other: &CellState) -> bool {
        self.0.matches(other) && self.1.matches(other)
    }
    fn vmatches(&self, other: &CellState) -> bool {
        self.0.vmatches(other) && self.1.vmatches(other)
    }
}
impl CellMatch for CellState {
    fn matches(&self, other: &CellState) -> bool {
        *self == *other
    }
    fn vmatches(&self, other: &CellState) -> bool {
        self.floor.vmatches(other) && self.obj.vmatches(other)
    }
}

impl CellState {
    pub fn update(&mut self, fill: impl Fill) {
        fill.fill(self);
    }
    pub fn matches(&self, other: impl CellMatch) -> bool {
        other.matches(self)
    }
    pub fn vmatches(&self, other: impl CellMatch) -> bool {
        other.vmatches(self)
    }
}

pub trait BoardArt {
    fn line(&mut self, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Fill);
    fn pt(&mut self, pt: impl Into<Coord>, fill: impl Fill);
    fn circle(&mut self, center: impl Into<Coord>, radius: usize, fill: impl Fill);
    fn rect(&mut self, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Fill);
    fn text(&mut self, text: &str, coord: impl Into<Coord>, fill: impl Fill, empty: impl Fill);
    fn explosion(&mut self, center: impl Into<Coord>, fill: impl Fill + CellMatch);
}
impl<const W: usize, const H: usize> BoardArt for Board<W, H> {
    fn line(&mut self, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Fill) {
        line(self, from, to, fill);
    }

    fn pt(&mut self, pt: impl Into<Coord>, fill: impl Fill) {
        let pt = pt.into();
        if !pt.in_bounds() { return; }
        self.cell_at_mut(pt).update(fill);
    }

    fn circle(&mut self, center: impl Into<Coord>, radius: usize, fill: impl Fill) {
        circle(self, center, radius, fill);
    }

    fn rect(&mut self, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Fill) {
        rect(self, from, to, fill);
    }

    fn text(&mut self, text: &str, coord: impl Into<Coord>, fill: impl Fill, empty: impl Fill) {
        let Coord { mut x, y } = coord.into();

        for mut letter in text.chars() {
            letter = letter.to_ascii_lowercase();
            if let Some(grid) = GRIDS.get(&letter) {
                write_letter(grid, x, y, self, fill, empty);
                x += C_WIDTH + 1;
            }
        }
    }

    fn explosion(&mut self, center: impl Into<Coord>, fill: impl Fill + CellMatch) {
        explosion(self, center, fill);
    }
}

pub fn write_letter<
    const C_W: usize,
    const C_H: usize
>(
    grid: &[[bool; C_W]; C_H],
    x: usize, y: usize,
    board: &mut (impl BoardArt + ?Sized),
    fill: impl Fill,
    empty: impl Fill
) {
    for (dy, row) in grid.iter().enumerate() {
        for (dx, should_fill) in row.iter().enumerate() {
            if *should_fill {
                board.pt((x + dx, y + dy), fill);
            } else {
                board.pt((x + dx, y + dy), empty);
            }
        }
        board.pt((x + row.len(), y + dy), empty);
    }
}

fn lerp(from: usize, to: usize, num: usize, den: usize) -> usize {
    (from * (den - num) + to * num) / den
}

fn lerp_coord(from: impl Into<Coord>, to: impl Into<Coord>, num: usize, den: usize) -> Coord {
    let (from, to) = (from.into(), to.into());
    Coord {
        x: lerp(from.x, to.x, num, den),
        y: lerp(from.y, to.y, num, den),
    }
}

fn dist(from: Coord, to: Coord) -> usize {
    let dx = from.x.abs_diff(to.x);
    let dy = from.y.abs_diff(to.y);
    let dist2 = (dx * dx + dy * dy) as f64;
    dist2.sqrt() as usize
}

fn line<const W: usize, const H: usize>(board: &mut Board<W, H>, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Fill) {
    let (from, to) = (from.into(), to.into());
    
    let n = dist(from, to) + 1;
    for i in 0..=n {
        let coord = lerp_coord(from, to, i, n);
        board.pt(coord, fill);
    }
}

fn circle<const W: usize, const H: usize>(board: &mut Board<W, H>, center: impl Into<Coord>, radius: usize, fill: impl Fill) {
    let center = center.into();

    let radius = radius as f64;
    let r2 = radius * radius;
    let ylim = (radius / f64::sqrt(2.0)) as usize + 1;
    for y in 0..ylim {
        let y2 = y * y;
        let y2 = y2 as f64;
        let x = f64::sqrt(r2 - y2);
        let x = x.round() as usize;

        // x >= y
        // We only need to check the negative boundary. 
        // `set_cell_at` checks the positive boundary.
        // Q1
        let mut coord;
        coord = (center.x + x, center.y + y);
        board.pt(coord, fill);

        coord = (center.x + y, center.y + x);
        board.pt(coord, fill);

        // Q2
        // whether we can use `x` in Q2 horizontally
        let q2x = x < center.x;
        // whether we can use `y` in Q2 horizontally
        let q2y = y < center.x;

        if q2x {
            coord = (center.x - x, center.y + y);
            board.pt(coord, fill);
        }
        if q2y {
            coord = (center.x - y, center.y + x);
            board.pt(coord, fill);
        }

        // Q4
        // whether we can use `x` in Q2 vertically
        let q4x = x < center.y;
        // whether we can use `y` in Q2 vertically
        let q4y = y < center.y;

        if q4x {
            coord = (center.x + y, center.y - x);
            board.pt(coord, fill);
        }
        if q4y {
            coord = (center.x + x, center.y - y);
            board.pt(coord, fill);
        }

        // Q3
        // whether we can use `x` in Q3 horizontally and `y` vertically
        let q3x = q2x && q4y;
        // whether we can use `y` in Q3 horizontally and `x` vertically
        let q3y = q2y && q4x;
        if q3x {
            coord = (center.x - x, center.y - y);
            board.pt(coord, fill);
        }
        if q3y {
            coord = (center.x - y, center.y - x);
            board.pt(coord, fill);
        }
    }
}

fn rect<const W: usize, const H: usize>(board: &mut Board<W, H>, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Fill) {
    const fn minmax(a: usize, b: usize) -> (usize, usize) {
        if a < b {
            (a, b)
        } else {
            (b, a)
        }
    }
    let (from, to) = (from.into(), to.into());
    let (x1, x2) = minmax(from.x, to.x);
    let (y1, y2) = minmax(from.y, to.y);
    for y in y1..y2 {
        for x in x1..x2 {
            board.pt((x, y), fill);
        }
    }
}

fn explosion<const W: usize, const H: usize>(board: &mut Board<W, H>, center: impl Into<Coord>, fill: impl Fill + CellMatch) {
    let center = center.into();
    for _ in 0..EXPLOSION_WALK_COUNT {
        walk(board, center, EXPLOSION_WALK_MAX_DIST, fill);
    }
}

fn walk<const W: usize, const H: usize>(board: &mut Board<W, H>, from: impl Into<Coord>, max_dist: usize, fill: impl Fill + CellMatch) {
    let mut pos = from.into();
    for _ in 0..max_dist {
        if !board.cell_at(pos).vmatches(fill) {
            // Do not overwrite a border
            if !board.cell_at(pos).vmatches(CellObject::Border) {
                board.pt(pos, fill);
            }
            break;
        }
        let dir = rand::random();
        pos = pos.add_wrapped(dir);
    }
}
