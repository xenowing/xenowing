mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use modules::*;
use rtl_meta::shovel::video_test_pattern_generator::*;

#[derive(Default)]
struct VideoTimingGenerator {
    x: u32,
    y: u32,

    fractional_cycles: u16,
}

impl VideoTimingGenerator {
    fn posedge_clk(&mut self) -> (bool, bool) {
        let mut vsync_pulse = false;
        let mut line_pulse = false;

        let (res, wrap) = self
            .fractional_cycles
            .overflowing_add(((1u64 << 16) * 12288 / 160000) as _);
        self.fractional_cycles = res;
        if wrap {
            if self.x == 0 {
                if self.y == 0 {
                    vsync_pulse = true;
                }

                if self.y >= VERTICAL_BACK_PORCH - 1
                    && self.y < ACTIVE_HEIGHT + VERTICAL_BACK_PORCH - 1
                {
                    line_pulse = true;
                }
            }

            self.x += 1;
            if self.x == TOTAL_WIDTH {
                self.x = 0;

                self.y += 1;
                if self.y == TOTAL_HEIGHT {
                    self.y = 0;
                }
            }
        }

        (vsync_pulse, line_pulse)
    }
}

fn main() {
    let mut window = Window::new(
        "strugl",
        ACTIVE_WIDTH as _,
        ACTIVE_HEIGHT as _,
        WindowOptions {
            scale: Scale::X2,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    let mut char_display_buffer = vec![false; (ACTIVE_WIDTH * ACTIVE_HEIGHT) as usize];
    let mut test_pattern_buffer = vec![0; (ACTIVE_WIDTH * ACTIVE_HEIGHT) as usize];

    let mut buffer = vec![0; (ACTIVE_WIDTH * ACTIVE_HEIGHT) as usize];

    let mut video_timing_generator = VideoTimingGenerator::default();

    let mut char_display = CharDisplay::new();
    char_display.reset();

    let mut video_test_pattern_generator = VideoTestPatternGenerator::new();
    video_test_pattern_generator.reset();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut char_display_buffer_pos = 0;
        let mut test_pattern_buffer_pos = 0;

        loop {
            let (vsync_pulse, line_pulse) = video_timing_generator.posedge_clk();

            char_display.system_write_vsync_pulse = vsync_pulse;
            char_display.system_write_line_pulse = line_pulse;

            char_display.prop();

            if char_display.video_line_buffer_write_enable {
                char_display_buffer[char_display_buffer_pos] =
                    char_display.video_line_buffer_write_data;
                char_display_buffer_pos += 1;
            }

            char_display.posedge_clk();

            video_test_pattern_generator.system_write_vsync_pulse = vsync_pulse;
            video_test_pattern_generator.system_write_line_pulse = line_pulse;

            video_test_pattern_generator.prop();

            if video_test_pattern_generator.video_line_buffer_write_enable {
                test_pattern_buffer[test_pattern_buffer_pos] =
                    video_test_pattern_generator.video_line_buffer_write_data & 0x00f8fcf8;
                test_pattern_buffer_pos += 1;
            }

            video_test_pattern_generator.posedge_clk();

            if vsync_pulse {
                break;
            }
        }

        for ((&char_display_pixel, &test_pattern_pixel), pixel) in char_display_buffer
            .iter()
            .zip(test_pattern_buffer.iter())
            .zip(buffer.iter_mut())
        {
            *pixel = if char_display_pixel {
                0xffffff
            } else {
                test_pattern_pixel
            };
        }

        window
            .update_with_buffer(&buffer, ACTIVE_WIDTH as _, ACTIVE_HEIGHT as _)
            .unwrap();
    }
}
