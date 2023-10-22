use kaze::*;

use rtl::pocket::video_test_pattern_generator::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let file = File::create(&dest_path).unwrap();

    let c = Context::new();

    let video_test_pattern_generator =
        VideoTestPatternGenerator::new("video_test_pattern_generator", &c);

    sim::generate(
        video_test_pattern_generator.m,
        sim::GenerationOptions::default(),
        file,
    )
}
