use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use image::{GenericImageView, Pixel, Rgba};
use into_color::{as_color, color_space};

fn main() {
    let floor_dir = Path::new("res/images/floor/");
    let elev_dir = Path::new("res/images/elev/");
    let fert_dir = Path::new("res/images/fert/");
    let output_dir = Path::new("res/levels/");
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    for floor_entry in fs::read_dir(floor_dir).expect("Failed to read input directory") {
        let floor_entry = floor_entry.expect("Failed to read directory entry");
        let floor_path = floor_entry.path();
        let mut elev_path = PathBuf::from(elev_dir);
        elev_path.push(floor_path.file_name().expect("Unable to get filename"));
        let mut fert_path = PathBuf::from(fert_dir);
        fert_path.push(floor_path.file_name().expect("Unable to get filename"));

        if floor_path.is_file() && elev_path.is_file() && fert_path.is_file() {
            let floor_img = image::open(&floor_path).expect("Failed to open floor image");
            let elev_img = image::open(&elev_path).expect("Failed to open elev image");
            let fert_img = image::open(&fert_path).expect("Failed to open fert image");
            let mut output = Vec::new();

            for ((pixel, elev), fert) in floor_img.pixels().zip(elev_img.pixels()).zip(fert_img.pixels()) {
                let color = pixel.2.to_rgba();
                let elev = elev.2.to_rgba();
                let fert = fert.2.to_rgba();
                let byte;
                if color == as_rgba!(EMPTY_COLOR) {
                    byte = 0x0; // Empty
                } else if color == as_rgba!(WATER_COLOR) {
                    byte = 0x1; // Water
                } else if color == as_rgba!(LAVA_COLOR) {
                    byte = 0x2; // Lava
                } else if color == as_rgba!(WALL_COLOR) {
                    byte = 0x3; // Wall
                } else if color == as_rgba!(BORDER_COLOR) {
                    byte = 0x4; // Border
                } else if color == as_rgba!(SEED_COLOR) {
                    byte = 0x5; // Seed
                } else if color == as_rgba!(COIN_COLOR) {
                    byte = 0x6;
                } else if color == as_rgba!(PM_COLOR) {
                    byte = 0x7;
                } else {
                    byte = 0x0;
                };
                output.push(byte);

                let elev = elev[0];
                output.push(elev);

                let fert = fert[0];
                output.push(fert);
            }

            let file_name = output_dir.join(floor_path.file_stem().unwrap()).with_extension("bin");
            let mut file = File::create(file_name).expect("Failed to create output file");
            file.write_all(&output).expect("Failed to write to output file");
        } else {
            println!("Unable to find all files for {:?}", floor_path.file_name().expect("Unable to get filename"));
        }
    }
}

type Color = (u8, u8, u8);

// Floor colors
const EMPTY_COLOR: Color = as_color!("#ffffff");
const WATER_COLOR: Color = as_color!("#3f38ff");
const LAVA_COLOR: Color = as_color!("#fcb103");

// Object colors
const WALL_COLOR: Color = as_color!("#000000");
const SEED_COLOR: Color = as_color!("#065e00");
const BORDER_COLOR: Color = as_color!("#42005e");

// Other colors
const COIN_COLOR: Color = as_color!("#bdb600");
const PM_COLOR: Color = as_color!("#62fa4b");

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