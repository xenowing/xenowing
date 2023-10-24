use image::GenericImageView;
use kaze::*;
use rtl_meta::shovel::video_test_pattern_generator::*;

use std::io::Cursor;

const CHAR_DIM: u32 = 8;

const CHARS_WIDTH: u32 = ACTIVE_WIDTH / CHAR_DIM;
const CHARS_HEIGHT: u32 = ACTIVE_HEIGHT / CHAR_DIM;

pub struct CharDisplay<'a> {
    pub m: &'a Module<'a>,

    pub system_write_vsync_pulse: &'a Input<'a>,
    pub system_write_line_pulse: &'a Input<'a>,

    pub video_line_buffer_write_enable: &'a Output<'a>,
    pub video_line_buffer_write_data: &'a Output<'a>,
}

impl<'a> CharDisplay<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> CharDisplay<'a> {
        let m = p.module(instance_name, "CharDisplay");

        let system_write_vsync_pulse = m.input("system_write_vsync_pulse", 1);
        let system_write_line_pulse = m.input("system_write_line_pulse", 1);

        // TODO: Expose read/write ports to make char and map data dynamic

        let font_image = image::io::Reader::new(Cursor::new(include_bytes!("font.png")))
            .with_guessed_format()
            .expect("Could not determine font image format")
            .decode()
            .expect("Could not decode font image");
        let (font_image_width, font_image_height) = font_image.dimensions();
        assert_eq!(font_image_width % CHAR_DIM, 0);
        assert_eq!(font_image_height % CHAR_DIM, 0);
        let mut font_data = Vec::new();
        for char_start_y in (0..font_image_height).step_by(CHAR_DIM as _) {
            for char_start_x in (0..font_image_width).step_by(CHAR_DIM as _) {
                for char_y in 0..CHAR_DIM {
                    let mut byte = 0;
                    for char_x in 0..CHAR_DIM {
                        let pixel =
                            font_image.get_pixel(char_start_x + char_x, char_start_y + char_y);
                        byte |= (((pixel[0] & 0xff) != 0) as u8) << (7 - char_x);
                    }
                    font_data.push(byte);
                }
            }
        }

        // TODO: Just use max here, and fill from software instead
        let char_mem_address_bit_width = (font_data.len() as f64).log2().ceil() as u32;
        let char_mem = m.mem("char_mem", char_mem_address_bit_width, 8);
        let mut char_mem_initial_contents = vec![0; 1 << char_mem_address_bit_width];
        char_mem_initial_contents[0..font_data.len()].copy_from_slice(&font_data);
        char_mem.initial_contents(&char_mem_initial_contents);

        let x = m.reg("x", 9);
        let y = m.reg("y", 8);

        let x_end = x.eq(m.lit(ACTIVE_WIDTH - 1, 9));

        let line_active = m.reg("line_active", 1);
        line_active.default_value(false);

        let pixel_buffer_addr = m.reg("pixel_buffer_addr", char_mem_address_bit_width);
        let pixel_buffer = m.reg("pixel_buffer", 8);

        x.drive_next(
            if_(system_write_line_pulse, m.lit(0u32, 9))
                .else_if(line_active, x + m.lit(1u32, 9))
                .else_(x),
        );

        y.drive_next(
            if_(system_write_vsync_pulse, m.lit(0u32, 8))
                .else_if(line_active & x_end, y + m.lit(1u32, 8))
                .else_(y),
        );

        line_active.drive_next(
            if_(system_write_line_pulse, m.lit(true, 1))
                .else_if(x_end, m.lit(false, 1))
                .else_(line_active),
        );

        pixel_buffer_addr.drive_next(
            if_(
                system_write_line_pulse,
                m.lit(0u32, char_mem_address_bit_width - 3)
                    .concat(y.bits(2, 0)),
            )
            .else_if(
                line_active & x.bits(2, 0).eq(m.lit(0u32, 3)),
                pixel_buffer_addr + m.lit(8u32, char_mem_address_bit_width),
            )
            .else_(pixel_buffer_addr),
        );

        let x_1 = x.reg_next("x_1");
        let line_active_1 = line_active.reg_next_with_default("line_active_1", false);
        let line_active_2 = line_active_1.reg_next_with_default("line_active_2", false);

        pixel_buffer.drive_next(
            if_(
                line_active_1,
                if_(
                    x_1.bits(2, 0).eq(m.lit(0u32, 3)),
                    char_mem.read_port(pixel_buffer_addr, line_active),
                )
                .else_(pixel_buffer.bits(6, 0).concat(m.lit(false, 1))),
            )
            .else_(pixel_buffer),
        );

        let video_line_buffer_write_data = pixel_buffer.bit(7);

        CharDisplay {
            m,

            system_write_vsync_pulse,
            system_write_line_pulse,

            video_line_buffer_write_enable: m
                .output("video_line_buffer_write_enable", line_active_2),
            video_line_buffer_write_data: m
                .output("video_line_buffer_write_data", video_line_buffer_write_data),
        }
    }
}
