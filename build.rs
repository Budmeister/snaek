use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use image::{GenericImageView, Pixel, Rgba};

fn main() {
    let input_dir = Path::new("res/images/");
    let output_dir = Path::new("res/levels/");
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    for entry in fs::read_dir(input_dir).expect("Failed to read input directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_file() {
            let img = image::open(&path).expect("Failed to open image");
            let mut output = Vec::new();

            for pixel in img.pixels() {
                let color = pixel.2.to_rgba();
                let byte = match color {
                    Rgba([0xFF, 0xFF, 0xFF, 0xFF]) => 0x0, // Empty
                    Rgba([0x3F, 0x38, 0xFF, 0xFF]) => 0x1, // Water
                    Rgba([0xFC, 0xB1, 0x03, 0xFF]) => 0x2, // Lava
                    Rgba([0x94, 0xFF, 0x8C, 0xFF]) => 0x3, // Turf
                    Rgba([0x00, 0x00, 0x00, 0xFF]) => 0x4, // Wall
                    Rgba([0x42, 0x00, 0x5E, 0xFF]) => 0x5, // Border
                    Rgba([0x06, 0x5e, 0x00, 0xFF]) => 0x6, // Seed
                    Rgba([0x69, 0x69, 0x69, 0xFF]) => 0x7, // Explosion indicator
                    _ => 0x0, // Default to Empty
                };
                output.push(byte);
            }

            let file_name = output_dir.join(path.file_stem().unwrap()).with_extension("bin");
            let mut file = File::create(file_name).expect("Failed to create output file");
            file.write_all(&output).expect("Failed to write to output file");
        }
    }
}
