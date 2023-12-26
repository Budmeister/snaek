use std::collections::HashMap;

use once_cell::sync::Lazy;


// Text consts
pub const C_WIDTH: usize = 3;
pub const C_HEIGHT: usize = 5;
pub type CharGrid = [[bool; C_WIDTH]; C_HEIGHT];

pub static GRIDS: Lazy<HashMap<char, CharGrid>> = Lazy::new(|| PRINTABLE_CHARS
        .chars()
        .zip(CHAR_GRIDS.into_iter())
        .collect());
static PRINTABLE_CHARS: &str = "abcdefghijklmnopqrstuvwxyz0123456789 :-(),.";
static CHAR_GRIDS: [CharGrid; 43] = {
    const X: bool = true;
    const O: bool = false;
    [
        // A
        [
            [O, X, O],
            [X, O, X],
            [X, X, X],
            [X, O, X],
            [X, O, X],
        ],
        // B
        [
            [X, X, O],
            [X, O, X],
            [X, X, O],
            [X, O, X],
            [X, X, O],
        ],
        // C
        [
            [O, X, X],
            [X, O, O],
            [X, O, O],
            [X, O, O],
            [O, X, X],
        ],
        // D
        [
            [X, X, O],
            [X, O, X],
            [X, O, X],
            [X, O, X],
            [X, X, O],
        ],
        // E
        [
            [X, X, X],
            [X, O, O],
            [X, X, O],
            [X, O, O],
            [X, X, X],
        ],
        // F
        [
            [X, X, X],
            [X, O, O],
            [X, X, O],
            [X, O, O],
            [X, O, O],
        ],
        // G
        [
            [O, X, X],
            [X, O, O],
            [X, O, O],
            [X, O, X],
            [O, X, X],
        ],
        // H
        [
            [X, O, X],
            [X, O, X],
            [X, X, X],
            [X, O, X],
            [X, O, X],
        ],
        // I
        [
            [X, X, X],
            [O, X, O],
            [O, X, O],
            [O, X, O],
            [X, X, X],
        ],
        // J
        [
            [X, X, X],
            [O, O, X],
            [O, O, X],
            [X, O, X],
            [O, X, X],
        ],
        // K
        [
            [X, O, X],
            [X, X, O],
            [X, O, O],
            [X, X, O],
            [X, O, X],
        ],
        // L
        [
            [X, O, O],
            [X, O, O],
            [X, O, O],
            [X, O, O],
            [X, X, X],
        ],
        // M
        [
            [X, O, X],
            [X, X, X],
            [X, X, X],
            [X, O, X],
            [X, O, X],
        ],
        // N
        [
            [X, X, O],
            [X, O, X],
            [X, O, X],
            [X, O, X],
            [X, O, X],
        ],
        // O
        [
            [O, X, O],
            [X, O, X],
            [X, O, X],
            [X, O, X],
            [O, X, O],
        ],
        // P
        [
            [X, X, O],
            [X, O, X],
            [X, X, O],
            [X, O, O],
            [X, O, O],
        ],
        // Q
        [
            [O, X, O],
            [X, O, X],
            [X, O, X],
            [O, X, O],
            [O, O, X],
        ],
        // R
        [
            [X, X, O],
            [X, O, X],
            [X, X, O],
            [X, O, X],
            [X, O, X],
        ],
        // S
        [
            [O, X, X],
            [X, O, O],
            [O, X, O],
            [O, O, X],
            [X, X, O],
        ],
        // T
        [
            [X, X, X],
            [O, X, O],
            [O, X, O],
            [O, X, O],
            [O, X, O],
        ],
        // U
        [
            [X, O, X],
            [X, O, X],
            [X, O, X],
            [X, O, X],
            [X, X, X],
        ],
        // V
        [
            [X, O, X],
            [X, O, X],
            [X, O, X],
            [X, O, X],
            [O, X, O],
        ],
        // W
        [
            [X, O, O],
            [X, O, X],
            [X, X, X],
            [X, X, X],
            [X, O, O],
        ],
        // X
        [
            [X, O, X],
            [X, O, X],
            [O, X, O],
            [X, O, X],
            [X, O, X],
        ],
        // Y
        [
            [X, O, X],
            [X, O, X],
            [O, X, O],
            [O, X, O],
            [O, X, O],
        ],
        // Z
        [
            [X, X, X],
            [O, O, X],
            [O, X, O],
            [X, O, O],
            [X, X, X],
        ],
        // 0
        [
            [O, X, O],
            [X, O, X],
            [X, O, X],
            [X, O, X],
            [O, X, O],
        ],
        // 1
        [
            [O, X, O],
            [X, X, O],
            [O, X, O],
            [O, X, O],
            [O, X, O],
        ],
        // 2
        [
            [X, X, O],
            [O, O, X],
            [O, O, X],
            [O, X, O],
            [X, X, X],
        ],
        // 3
        [
            [X, X, X],
            [O, O, X],
            [O, X, O],
            [O, O, X],
            [X, X, X],
        ],
        // 4
        [
            [X, O, X],
            [X, O, X],
            [X, X, X],
            [O, O, X],
            [O, O, X],
        ],
        // 5
        [
            [X, X, X],
            [X, O, O],
            [X, X, O],
            [O, O, X],
            [X, X, O],
        ],
        // 6
        [
            [O, X, X],
            [X, O, O],
            [X, X, O],
            [X, O, X],
            [O, X, O],
        ],
        // 7
        [
            [X, X, X],
            [O, O, X],
            [O, X, O],
            [O, X, O],
            [O, X, O],
        ],
        // 8
        [
            [X, X, X],
            [X, O, X],
            [X, X, X],
            [X, O, X],
            [X, X, X],
        ],
        // 9
        [
            [O, X, O],
            [X, O, X],
            [O, X, X],
            [O, O, X],
            [X, X, O],
        ],
        // <space>
        [
            [O, O, O],
            [O, O, O],
            [O, O, O],
            [O, O, O],
            [O, O, O],
        ],
        // :
        [
            [O, O, O],
            [O, X, O],
            [O, O, O],
            [O, X, O],
            [O, O, O],
        ],
        // -
        [
            [O, O, O],
            [O, O, O],
            [X, X, X],
            [O, O, O],
            [O, O, O],
        ],
        // (
        [
            [O, X, O],
            [X, O, O],
            [X, O, O],
            [X, O, O],
            [O, X, O],
        ],
        // )
        [
            [O, X, O],
            [O, O, X],
            [O, O, X],
            [O, O, X],
            [O, X, O],
        ],
        // ,
        [
            [O, O, O],
            [O, O, O],
            [O, X, O],
            [O, X, O],
            [X, O, O],
        ],
        // .
        [
            [O, O, O],
            [O, O, O],
            [O, O, O],
            [O, O, O],
            [O, X, O],
        ]
    ]
};
