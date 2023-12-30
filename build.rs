use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use image::{GenericImageView, Pixel, Rgba};
use into_color::{as_color, color_space};

fn main() {
    let floor_dir = Path::new("res/images/floor/");
    let elev_dir = Path::new("res/images/elev/");
    let output_dir = Path::new("res/levels/");
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    for floor_entry in fs::read_dir(floor_dir).expect("Failed to read input directory") {
        let floor_entry = floor_entry.expect("Failed to read directory entry");
        let floor_path = floor_entry.path();
        let mut elev_path = PathBuf::from(elev_dir);
        elev_path.push(floor_path.file_name().expect("Unable to get filename"));

        if floor_path.is_file() && elev_path.is_file() {
            let floor_img = image::open(&floor_path).expect("Failed to open floor image");
            let elev_img = image::open(&elev_path).expect("Failed to open elev image");
            let mut output = Vec::new();

            for (pixel, elev) in floor_img.pixels().zip(elev_img.pixels()) {
                let color = pixel.2.to_rgba();
                let elev = elev.2.to_rgba();
                let byte;
                if color == as_rgba!(EMPTY_COLOR) {
                    byte = 0x0; // Empty
                } else if color == as_rgba!(WATER_COLOR) {
                    byte = 0x1; // Water
                } else if color == as_rgba!(LAVA_COLOR) {
                    byte = 0x2; // Lava
                } else if color == as_rgba!(TURF_COLOR) {
                    byte = 0x3; // Turf
                } else if color == as_rgba!(WALL_COLOR) {
                    byte = 0x4; // Wall
                } else if color == as_rgba!(BORDER_COLOR) {
                    byte = 0x5; // Border
                } else if color == as_rgba!(SEED_COLOR) {
                    byte = 0x6; // Seed
                } else if color == as_rgba!(EXPLOSIVE_COLOR) {
                    byte = 0x7; // Explosion indicator
                } else if color == as_rgba!(DIRT_COLOR) {
                    byte = 0x8; // Dirt indicator
                } else {
                    byte = 0x0;
                };
                output.push(byte);

                let elev = elev[0];
                output.push(elev);
            }

            let file_name = output_dir.join(floor_path.file_stem().unwrap()).with_extension("bin");
            let mut file = File::create(file_name).expect("Failed to create output file");
            file.write_all(&output).expect("Failed to write to output file");
        }
    }
}

type Color = (u8, u8, u8);

// Floor colors
const EMPTY_COLOR: Color = as_color!("#ffffff");
const WATER_COLOR: Color = as_color!("#3f38ff");
const LAVA_COLOR: Color = as_color!("#fcb103");
const TURF_COLOR: Color = as_color!("#94ff8c");

// Object colors
const WALL_COLOR: Color = as_color!("#000000");
const FOOD_COLOR: Color = as_color!("#11ff00");
const SEED_COLOR: Color = as_color!("#065e00");
const SEED_COLORS: [Color; 8] = [
    as_color!("#2a5e00"),
    as_color!("#2a5904"),
    as_color!("#3a5904"),
    as_color!("#455904"),
    as_color!("#595904"),
    as_color!("#59540b"),
    as_color!("#594e04"),
    as_color!("#5e5200"),
];
const DEAD_SEED_COLOR: Color = as_color!("#542d1c");
const BORDER_COLOR: Color = as_color!("#42005e");
const SNAKE_COLOR_LIGHT_RED: Color = as_color!("#ff6038");
const SNAKE_COLOR_DARK_RED: Color = as_color!("#871d03");
const SNAKE_COLOR_HEAD: Color = as_color!("#eb9b2d");
const SNAKE_COLOR_HEAD_WITH_INVINC: Color = as_color!("#f8ffbd");

// Powerup colors
const EXPLOSIVE_COLOR: Color = as_color!("#696969");
const INVINC_COLOR: Color = as_color!("#000000");

// Other colors
const DIRT_COLOR: Color = as_color!("#422417");

// Terrain colors
sized_color_space!{
    TERRAIN_COLORS = [
        ("#000000", 0.0),
        ("#008d71", 0.45),
        ("#7b3c02", 0.56),
        ("#bab783", 0.58),
        ("#007103", 0.6),
        ("#828282", 0.85),
        ("#ffffff", 1.0)
    ],
    NUM_TERRAIN_COLORS = 256
}

mod macros {
    #[macro_export]
    macro_rules! sized_color_space {
        ($name:ident = [ $( $color:expr ),* ], $len_name:ident = $len:literal) => {
            const $name: [(u8, u8, u8); $len] = color_space!([ $( $color ),* ], $len);
            const $len_name: usize = $len;
        };
    }

    #[macro_export]
    macro_rules! as_rgba {
        ($color:expr) => {
            Rgba([$color.0, $color.1, $color.2, 0xFF])
        };
    }
}