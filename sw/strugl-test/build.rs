use qoi::*;

use image::GenericImageView;

use std::env;
use std::fs;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let input_file_name = "myface.png";
    let input = {
        let input = image::open(input_file_name).expect("Couldn't load image");
        let width = input.width();
        let height = input.height();
        let mut data = vec![Pixel::default(); (width * height) as usize].into_boxed_slice();
        for y in 0..height {
            for x in 0..width {
                let pixel = input.get_pixel(x, y);
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let a = pixel[3];
                data[(y * width + x) as usize] = Pixel::from_components(a, r, g, b);
            }
        }
        Image {
            width,
            height,
            data,
        }
    };
    let encoded = input.encode();

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("myface.bin");
    fs::write(dest_path, encoded)?;

    println!("cargo:rerun-if-changed={}", input_file_name);

    Ok(())
}
