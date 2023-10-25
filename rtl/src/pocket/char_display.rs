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

        // TODO: Fill from software instead
        // TODO: Get rid of magic numbers and move these to constants
        let char_mem_address_bit_width = 7 + 3;
        let char_mem = m.mem("char_mem", char_mem_address_bit_width, 8);
        let mut char_mem_initial_contents = vec![0; 1 << char_mem_address_bit_width];
        char_mem_initial_contents[0..font_data.len()].copy_from_slice(&font_data);
        char_mem.initial_contents(&char_mem_initial_contents);

        // TODO: Fill from software instead
        let map_mem_address_bit_width = ((CHARS_WIDTH * CHARS_HEIGHT) as f64).log2().ceil() as u32;
        let map_mem = m.mem("map_mem", map_mem_address_bit_width, 7);
        let mut map_mem_initial_contents = vec![0; 1 << map_mem_address_bit_width];
        let map_data_str = "Hello from char mem ROM! ...need to replace this with RAM ofc and fill it from CPU and whatever but this is cool for now.";
        let map_data_bytes = map_data_str.bytes().map(|x| x - 32).collect::<Vec<_>>();
        map_mem_initial_contents[0..map_data_bytes.len()].copy_from_slice(&map_data_bytes);
        map_mem.initial_contents(&map_mem_initial_contents);

        let x = m.reg("x", 9);
        let y = m.reg("y", 8);

        let x_end = x.eq(m.lit(ACTIVE_WIDTH - 1, 9));

        let line_active = m.reg("line_active", 1);
        line_active.default_value(false);

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

        let map_line_addr = m.reg("map_line_addr", map_mem_address_bit_width);
        map_line_addr.drive_next(
            if_(
                system_write_vsync_pulse,
                m.lit(0u32, map_mem_address_bit_width),
            )
            .else_if(
                line_active & x_end & y.bits(2, 0).eq(m.lit(7u32, 3)),
                map_line_addr + m.lit(CHARS_WIDTH, map_mem_address_bit_width),
            )
            .else_(map_line_addr),
        );

        let map_addr = m.reg("map_addr", map_mem_address_bit_width);
        map_addr.drive_next(
            if_(system_write_line_pulse, map_line_addr.into())
                .else_if(
                    line_active & x.bits(2, 0).eq(m.lit(7u32, 3)),
                    map_addr + m.lit(1u32, map_mem_address_bit_width),
                )
                .else_(map_addr),
        );

        let x_1 = x.reg_next("x_1");
        let x_2 = x_1.reg_next("x_2");
        let y_1 = y.reg_next("y_1");
        let line_active_1 = line_active.reg_next_with_default("line_active_1", false);
        let line_active_2 = line_active_1.reg_next_with_default("line_active_2", false);
        let line_active_3 = line_active_2.reg_next_with_default("line_active_3", false);

        let read_map = line_active & x.bits(2, 0).eq(m.lit(0u32, 3));
        let map_1 = map_mem.read_port(map_addr, read_map);
        let read_char_1 = line_active_1 & x_1.bits(2, 0).eq(m.lit(0u32, 3));
        let char_2 = char_mem.read_port(map_1.concat(y_1.bits(2, 0)), read_char_1);

        let pixel_buffer_3 = m.reg("pixel_buffer", 8);
        pixel_buffer_3.drive_next(
            if_(
                line_active_2,
                if_(x_2.bits(2, 0).eq(m.lit(0u32, 3)), char_2)
                    .else_(pixel_buffer_3.bits(6, 0).concat(m.lit(false, 1))),
            )
            .else_(pixel_buffer_3),
        );

        let video_line_buffer_write_data_3 = pixel_buffer_3.bit(7);

        CharDisplay {
            m,

            system_write_vsync_pulse,
            system_write_line_pulse,

            video_line_buffer_write_enable: m
                .output("video_line_buffer_write_enable", line_active_3),
            video_line_buffer_write_data: m.output(
                "video_line_buffer_write_data",
                video_line_buffer_write_data_3,
            ),
        }
    }
}
