use super::types::{Coord, Board, CellState, BoardExt};

use crate::text::{GRIDS, C_WIDTH, CharGrid};

pub trait BoardArtExt {
    fn line(&mut self, from: impl Into<Coord>, to: impl Into<Coord>);
    fn line_off(&mut self, from: impl Into<Coord>, to: impl Into<Coord>);
    fn pt(&mut self, pt: impl Into<Coord>);
    fn pt_off(&mut self, pt: impl Into<Coord>);
    fn circle(&mut self, center: impl Into<Coord>, radius: usize);
    fn circle_off(&mut self, center: impl Into<Coord>, radius: usize);
    fn text(&mut self, text: &str, coord: impl Into<Coord>);
}
impl BoardArtExt for Board {
    fn line(&mut self, from: impl Into<Coord>, to: impl Into<Coord>) {
        line(self, from, to, CellState::Filled);
    }

    fn line_off(&mut self, from: impl Into<Coord>, to: impl Into<Coord>) {
        line(self, from, to, CellState::Empty);
    }

    fn pt(&mut self, pt: impl Into<Coord>) {
        self.set_cell_at(pt, CellState::Filled);
    }

    fn pt_off(&mut self, pt: impl Into<Coord>) {
        self.set_cell_at(pt, CellState::Empty);
    }

    fn circle(&mut self, center: impl Into<Coord>, radius: usize) {
        circle(self, center, radius, CellState::Filled);
    }

    fn circle_off(&mut self, center: impl Into<Coord>, radius: usize) {
        circle(self, center, radius, CellState::Empty);
    }

    fn text(&mut self, text: &str, coord: impl Into<Coord>) {
        let Coord { mut x, y } = coord.into();
        for mut letter in text.chars() {
            letter = letter.to_ascii_lowercase();
            if let Some(grid) = GRIDS.get(&letter) {
                write_letter(grid, x, y, self);
                x += C_WIDTH + 1;
            }
        }
    }
}

fn write_letter(grid: &CharGrid, x: usize, y: usize, board: &mut Board) {
    for (dy, row) in grid.iter().enumerate() {
        for (dx, fill) in row.iter().enumerate() {
            board.set_cell_at((x + dx, y + dy), if *fill { CellState::Filled } else { CellState::Empty });
        }
        board.set_cell_at((x + row.len(), y + dy), CellState::Empty);
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

fn line(board: &mut Board, from: impl Into<Coord>, to: impl Into<Coord>, fill: CellState) {
    let (from, to) = (from.into(), to.into());
    let n = dist(from, to) + 1;
    for i in 0..=n {
        let coord = lerp_coord(from, to, i, n);
        board.set_cell_at(coord, fill);
    }
}

fn circle(board: &mut Board, center: impl Into<Coord>, radius: usize, fill: CellState) {
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
        board.set_cell_at(coord, fill);

        coord = (center.x + y, center.y + x);
        board.set_cell_at(coord, fill);

        // Q2
        // whether we can use `x` in Q2 horizontally
        let q2x = x < center.x;
        // whether we can use `y` in Q2 horizontally
        let q2y = y < center.x;

        if q2x {
            coord = (center.x - x, center.y + y);
            board.set_cell_at(coord, fill);
        }
        if q2y {
            coord = (center.x - y, center.y + x);
            board.set_cell_at(coord, fill);
        }

        // Q4
        // whether we can use `x` in Q2 vertically
        let q4x = x < center.y;
        // whether we can use `y` in Q2 vertically
        let q4y = y < center.y;

        if q4x {
            coord = (center.x + y, center.y - x);
            board.set_cell_at(coord, fill);
        }
        if q4y {
            coord = (center.x + x, center.y - y);
            board.set_cell_at(coord, fill);
        }

        // Q3
        // whether we can use `x` in Q3 horizontally and `y` vertically
        let q3x = q2x && q4y;
        // whether we can use `y` in Q3 horizontally and `x` vertically
        let q3y = q2y && q4x;
        if q3x {
            coord = (center.x - x, center.y - y);
            board.set_cell_at(coord, fill);
        }
        if q3y {
            coord = (center.x - y, center.y - x);
            board.set_cell_at(coord, fill);
        }
    }
}