use super::types::{Coord, Board, CellState, CellFloor, CellObject};

use crate::text::{GRIDS, C_WIDTH, CharGrid};

const EXPLOSION_WALK_COUNT: usize = 30;
const EXPLOSION_WALK_MAX_DIST: usize = 30;

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub struct Fill {
    floor: Option<CellFloor>,
    obj: Option<CellObject>,
    elev: Option<u8>
}
impl From<CellFloor> for Fill {
    fn from(value: CellFloor) -> Self {
        Self {
            floor: Some(value),
            obj: None,
            elev: None,
        }
    }
}
impl From<CellObject> for Fill {
    fn from(value: CellObject) -> Self {
        Self {
            floor: None,
            obj: Some(value),
            elev: None,
        }
    }
}
impl From<u8> for Fill {
    fn from(value: u8) -> Self {
        Self {
            floor: None,
            obj: None,
            elev: Some(value),
        }
    }
}
impl From<CellState> for Fill {
    fn from(value: CellState) -> Self {
        Self {
            floor: Some(value.floor),
            obj: Some(value.obj),
            elev: Some(value.elev),
        }
    }
}
impl From<()> for Fill {
    fn from(_: ()) -> Self {
        Self {
            floor: None,
            obj: None,
            elev: None,
        }
    }
}
impl Fill {
    pub fn update(&mut self, other: Fill) {
        self.floor = other.floor.or(self.floor);
        self.obj = other.obj.or(self.obj);
        self.elev = other.elev.or(self.elev);
    }
}
impl<F1: Into<Fill>, F2: Into<Fill>> From<(F1, F2)> for Fill {
    fn from(value: (F1, F2)) -> Self {
        let (f1, f2) = value;
        let (mut f1, f2) = (f1.into(), f2.into());
        f1.update(f2);
        f1
    }
}
impl CellState {
    pub fn update(&mut self, fill: impl Into<Fill>) {
        let fill = fill.into();
        if let Some(floor) = fill.floor {
            self.floor = floor;
        }
        if let Some(obj) = fill.obj {
            self.obj = obj;
        }
        if let Some(elev) = fill.elev {
            self.elev = elev
        }
    }
    pub fn matches(&self, other: impl Into<Fill>) -> bool {
        let other = other.into();
        if let Some(floor) = other.floor {
            if self.floor != floor {
                return false;
            }
        }
        if let Some(obj) = other.obj {
            if self.obj != obj {
                return false;
            }
        }
        if let Some(elev) = other.elev {
            if self.elev != elev {
                return false;
            }
        }
        true
    }
}

pub trait BoardArt {
    fn line(&mut self, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Into<Fill>);
    fn pt(&mut self, pt: impl Into<Coord>, fill: impl Into<Fill>);
    fn circle(&mut self, center: impl Into<Coord>, radius: usize, fill: impl Into<Fill>);
    fn rect(&mut self, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Into<Fill>);
    fn text(&mut self, text: &str, coord: impl Into<Coord>, fill: impl Into<Fill>, empty: impl Into<Fill>);
    fn explosion(&mut self, center: impl Into<Coord>, fill: impl Into<Fill>);
}
impl<const W: usize, const H: usize> BoardArt for Board<W, H> {
    fn line(&mut self, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Into<Fill>) {
        line(self, from, to, fill);
    }

    fn pt(&mut self, pt: impl Into<Coord>, fill: impl Into<Fill>) {
        let pt = pt.into();
        if !pt.in_bounds() { return; }
        self.cell_at_mut(pt).update(fill);
    }

    fn circle(&mut self, center: impl Into<Coord>, radius: usize, fill: impl Into<Fill>) {
        circle(self, center, radius, fill);
    }

    fn rect(&mut self, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Into<Fill>) {
        rect(self, from, to, fill);
    }

    fn text(&mut self, text: &str, coord: impl Into<Coord>, fill: impl Into<Fill>, empty: impl Into<Fill>) {
        let Coord { mut x, y } = coord.into();
        let (fill, empty) = (fill.into(), empty.into());

        for mut letter in text.chars() {
            letter = letter.to_ascii_lowercase();
            if let Some(grid) = GRIDS.get(&letter) {
                write_letter(grid, x, y, self, fill, empty);
                x += C_WIDTH + 1;
            }
        }
    }

    fn explosion(&mut self, center: impl Into<Coord>, fill: impl Into<Fill>) {
        explosion(self, center, fill);
    }
}

fn write_letter<const W: usize, const H: usize>(grid: &CharGrid, x: usize, y: usize, board: &mut Board<W, H>, fill: impl Into<Fill>, empty: impl Into<Fill>) {
    let (fill, empty) = (fill.into(), empty.into());
    for (dy, row) in grid.iter().enumerate() {
        for (dx, should_fill) in row.iter().enumerate() {
            board.pt((x + dx, y + dy), if *should_fill { fill } else { empty });
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

fn line<const W: usize, const H: usize>(board: &mut Board<W, H>, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Into<Fill>) {
    let (from, to) = (from.into(), to.into());
    let fill = fill.into();
    
    let n = dist(from, to) + 1;
    for i in 0..=n {
        let coord = lerp_coord(from, to, i, n);
        board.pt(coord, fill);
    }
}

fn circle<const W: usize, const H: usize>(board: &mut Board<W, H>, center: impl Into<Coord>, radius: usize, fill: impl Into<Fill>) {
    let center = center.into();
    let fill = fill.into();

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

fn rect<const W: usize, const H: usize>(board: &mut Board<W, H>, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Into<Fill>) {
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
    let fill = fill.into();
    for y in y1..y2 {
        for x in x1..x2 {
            board.pt((x, y), fill);
        }
    }
}

fn explosion<const W: usize, const H: usize>(board: &mut Board<W, H>, center: impl Into<Coord>, fill: impl Into<Fill>) {
    let (center, fill) = (center.into(), fill.into());
    for _ in 0..EXPLOSION_WALK_COUNT {
        walk(board, center, EXPLOSION_WALK_MAX_DIST, fill);
    }
}

fn walk<const W: usize, const H: usize>(board: &mut Board<W, H>, from: impl Into<Coord>, max_dist: usize, fill: impl Into<Fill>) {
    let mut pos = from.into();
    let fill = fill.into();
    for _ in 0..max_dist {
        if !board.cell_at(pos).matches(fill) {
            // Do not overwrite a border
            if !board.cell_at(pos).matches(CellObject::Border) {
                board.pt(pos, fill);
            }
            break;
        }
        let dir = rand::random();
        pos = pos.add_wrapped(dir);
    }
}
