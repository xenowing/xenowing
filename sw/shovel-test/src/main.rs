mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use modules::*;
use rtl_meta::shovel::video_test_pattern_generator::*;

fn main() {
    let mut window = Window::new(
        "strugl",
        WIDTH as _,
        HEIGHT as _,
        WindowOptions {
            scale: Scale::X2,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    let mut buffer = vec![0; (WIDTH * HEIGHT) as usize];

    let mut video_test_pattern_generator = VideoTestPatternGenerator::new();
    video_test_pattern_generator.reset();
    video_test_pattern_generator.system_write_reset_pulse = false;
    video_test_pattern_generator.system_write_line_pulse = false;

    let cycle_period_ns = 6u64;
    let frames_per_s = 60u64;
    let ns_per_s = 1_000_000_000u64;
    let ns_per_frame = ns_per_s / frames_per_s;
    let cycles_per_frame = ns_per_frame / cycle_period_ns;
    println!("cycles per frame: {}", cycles_per_frame);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Reset pulse
        video_test_pattern_generator.system_write_reset_pulse = true;
        video_test_pattern_generator.prop();
        video_test_pattern_generator.posedge_clk();
        video_test_pattern_generator.system_write_reset_pulse = false;

        for y in 0..HEIGHT {
            // Line pulse
            video_test_pattern_generator.system_write_line_pulse = true;
            video_test_pattern_generator.prop();
            video_test_pattern_generator.posedge_clk();
            video_test_pattern_generator.system_write_line_pulse = false;

            for x in 0..WIDTH {
                video_test_pattern_generator.prop();
                assert!(video_test_pattern_generator.video_line_buffer_write_enable);
                buffer[(y * WIDTH + x) as usize] =
                    video_test_pattern_generator.video_line_buffer_write_data & 0x00f8fcf8;
                video_test_pattern_generator.posedge_clk();
            }
        }

        window
            .update_with_buffer(&buffer, WIDTH as _, HEIGHT as _)
            .unwrap();
    }
}
