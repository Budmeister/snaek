use super::types::{Coord, Board, CellState, CellFloor, CellObject};

use crate::text::{GRIDS, C_WIDTH, CharGrid};

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
enum Fill {
    Floor(CellFloor),
    Object(CellObject),
    Both(CellState),
}
impl From<CellFloor> for Fill {
    fn from(value: CellFloor) -> Self {
        Self::Floor(value)
    }
}
impl From<CellObject> for Fill {
    fn from(value: CellObject) -> Self {
        Self::Object(value)
    }
}
impl From<CellState> for Fill {
    fn from(value: CellState) -> Self {
        Self::Both(value)
    }
}
impl CellState {
    fn update(&mut self, fill: impl Into<Fill>) {
        match fill.into() {
            Fill::Floor(floor) => self.floor = floor,
            Fill::Object(obj) => self.obj = obj,
            Fill::Both(state) => *self = state,
        }
    }
}

pub trait BoardArt {
    fn line(&mut self, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Into<Fill>);
    fn pt(&mut self, pt: impl Into<Coord>, fill: impl Into<Fill>);
    fn circle(&mut self, center: impl Into<Coord>, radius: usize, fill: impl Into<Fill>);
    fn text(&mut self, text: &str, coord: impl Into<Coord>, fill: impl Into<Fill>, empty: impl Into<Fill>);
}
impl BoardArt for Board {
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
}

fn write_letter(grid: &CharGrid, x: usize, y: usize, board: &mut Board, fill: impl Into<Fill>, empty: impl Into<Fill>) {
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

fn line(board: &mut Board, from: impl Into<Coord>, to: impl Into<Coord>, fill: impl Into<Fill>) {
    let (from, to) = (from.into(), to.into());
    let fill = fill.into();
    
    let n = dist(from, to) + 1;
    for i in 0..=n {
        let coord = lerp_coord(from, to, i, n);
        board.pt(coord, fill);
    }
}

fn circle(board: &mut Board, center: impl Into<Coord>, radius: usize, fill: impl Into<Fill>) {
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