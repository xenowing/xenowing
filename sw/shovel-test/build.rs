use kaze::*;

use rtl::bootloader::*;
use rtl::pocket::shovel::*;
use rtl::pocket::video_test_pattern_generator::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let mut file = File::create(&dest_path).unwrap();

    let c = Context::new();

    let top = c.module("top", "Top");
    let shovel = Shovel::new("shovel", top);
    shovel
        .system_write_vsync_pulse
        .drive(top.input("system_write_vsync_pulse", 1));
    shovel
        .system_write_line_pulse
        .drive(top.input("system_write_line_pulse", 1));
    top.output(
        "video_line_buffer_write_enable",
        shovel.video_line_buffer_write_enable,
    );
    top.output(
        "video_line_buffer_write_data",
        shovel.video_line_buffer_write_data,
    );
    let bootloader = Bootloader::new("bootloader", top);
    shovel.bootloader.connect(&bootloader.client_port);

    let video_test_pattern_generator =
        VideoTestPatternGenerator::new("video_test_pattern_generator", &c);

    sim::generate(top, sim::GenerationOptions::default(), &mut file)?;

    sim::generate(
        video_test_pattern_generator.m,
        sim::GenerationOptions::default(),
        file,
    )
}
