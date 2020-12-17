use crate::approx_reciprocal;
use crate::word_mem::*;

use kaze::*;

pub const TILE_DIM_BITS: u32 = 4;
pub const TILE_DIM: u32 = 1 << TILE_DIM_BITS;
pub const TILE_PIXELS_BITS: u32 = TILE_DIM_BITS * 2;
pub const TILE_PIXELS: u32 = 1 << TILE_PIXELS_BITS;
pub const TILE_PIXELS_WORDS_BITS: u32 = TILE_PIXELS_BITS - 2;

pub const TEX_DIM_BITS: u32 = 4;
pub const TEX_DIM: u32 = 1 << TEX_DIM_BITS;
pub const TEX_PIXELS_BITS: u32 = TEX_DIM_BITS * 2;
pub const TEX_PIXELS: u32 = 1 << TEX_PIXELS_BITS;
pub const TEX_PIXELS_WORDS_BITS: u32 = TEX_PIXELS_BITS - 2;
pub const TEX_BUFFER_DIM_BITS: u32 = TEX_DIM_BITS - 1;
pub const TEX_BUFFER_DIM: u32 = 1 << TEX_BUFFER_DIM_BITS;
pub const TEX_BUFFER_PIXELS_BITS: u32 = TEX_BUFFER_DIM_BITS * 2;
pub const TEX_BUFFER_PIXELS: u32 = 1 << TEX_BUFFER_PIXELS_BITS;

pub const EDGE_FRACT_BITS: u32 = 8;
pub const COLOR_WHOLE_BITS: u32 = 9;
pub const COLOR_FRACT_BITS: u32 = 12;
pub const W_INVERSE_FRACT_BITS: u32 = 30;
pub const Z_FRACT_BITS: u32 = 30; // Must be greater than 16
pub const ST_FRACT_BITS: u32 = 24;
pub const ST_FILTER_FRACT_BITS: u32 = 4; // Must be less than ST_FRACT_BITS
pub const RESTORED_W_FRACT_BITS: u32 = 8; // Must be less than W_INVERSE_FRACT_BITS and ST_FRACT_BITS

pub const REG_BUS_ADDR_BIT_WIDTH: u32 = 6;

pub const REG_STATUS_ADDR: u32 = 0;
pub const REG_START_ADDR: u32 = 0;

pub const REG_DEPTH_SETTINGS_ADDR: u32 = 1;
pub const REG_DEPTH_SETTINGS_BITS: u32 = 2;
pub const REG_DEPTH_TEST_ENABLE_BIT: u32 = 0;
pub const REG_DEPTH_WRITE_MASK_ENABLE_BIT: u32 = 1;

pub const REG_TEXTURE_SETTINGS_ADDR: u32 = 2;
pub const REG_TEXTURE_SETTINGS_BITS: u32 = 1;
pub const REG_TEXTURE_SETTINGS_FILTER_SELECT_BIT: u32 = 0;
pub const REG_TEXTURE_SETTINGS_FILTER_SELECT_NEAREST: u32 = 0;
pub const REG_TEXTURE_SETTINGS_FILTER_SELECT_BILINEAR: u32 = 1;

pub const REG_BLEND_SETTINGS_ADDR: u32 = 3;
pub const REG_BLEND_SETTINGS_BITS: u32 = 4;
pub const REG_BLEND_SETTINGS_SRC_FACTOR_BIT_OFFSET: u32 = 0;
pub const REG_BLEND_SETTINGS_SRC_FACTOR_BITS: u32 = 2;
pub const REG_BLEND_SETTINGS_SRC_FACTOR_ZERO: u32 = 0;
pub const REG_BLEND_SETTINGS_SRC_FACTOR_ONE: u32 = 1;
pub const REG_BLEND_SETTINGS_SRC_FACTOR_SRC_ALPHA: u32 = 2;
pub const REG_BLEND_SETTINGS_SRC_FACTOR_ONE_MINUS_SRC_ALPHA: u32 = 3;
pub const REG_BLEND_SETTINGS_DST_FACTOR_BIT_OFFSET: u32 = REG_BLEND_SETTINGS_SRC_FACTOR_BIT_OFFSET + REG_BLEND_SETTINGS_SRC_FACTOR_BITS;
pub const REG_BLEND_SETTINGS_DST_FACTOR_BITS: u32 = 2;
pub const REG_BLEND_SETTINGS_DST_FACTOR_ZERO: u32 = 0;
pub const REG_BLEND_SETTINGS_DST_FACTOR_ONE: u32 = 1;
pub const REG_BLEND_SETTINGS_DST_FACTOR_SRC_ALPHA: u32 = 2;
pub const REG_BLEND_SETTINGS_DST_FACTOR_ONE_MINUS_SRC_ALPHA: u32 = 3;

pub const REG_W0_MIN_ADDR: u32 = 4;
pub const REG_W0_DX_ADDR: u32 = 5;
pub const REG_W0_DY_ADDR: u32 = 6;
pub const REG_W1_MIN_ADDR: u32 = 7;
pub const REG_W1_DX_ADDR: u32 = 8;
pub const REG_W1_DY_ADDR: u32 = 9;
pub const REG_W2_MIN_ADDR: u32 = 10;
pub const REG_W2_DX_ADDR: u32 = 11;
pub const REG_W2_DY_ADDR: u32 = 12;
pub const REG_R_MIN_ADDR: u32 = 13;
pub const REG_R_DX_ADDR: u32 = 14;
pub const REG_R_DY_ADDR: u32 = 15;
pub const REG_G_MIN_ADDR: u32 = 16;
pub const REG_G_DX_ADDR: u32 = 17;
pub const REG_G_DY_ADDR: u32 = 18;
pub const REG_B_MIN_ADDR: u32 = 19;
pub const REG_B_DX_ADDR: u32 = 20;
pub const REG_B_DY_ADDR: u32 = 21;
pub const REG_A_MIN_ADDR: u32 = 22;
pub const REG_A_DX_ADDR: u32 = 23;
pub const REG_A_DY_ADDR: u32 = 24;
pub const REG_W_INVERSE_MIN_ADDR: u32 = 25;
pub const REG_W_INVERSE_DX_ADDR: u32 = 26;
pub const REG_W_INVERSE_DY_ADDR: u32 = 27;
pub const REG_Z_MIN_ADDR: u32 = 28;
pub const REG_Z_DX_ADDR: u32 = 29;
pub const REG_Z_DY_ADDR: u32 = 30;
pub const REG_S_MIN_ADDR: u32 = 31;
pub const REG_S_DX_ADDR: u32 = 32;
pub const REG_S_DY_ADDR: u32 = 33;
pub const REG_T_MIN_ADDR: u32 = 34;
pub const REG_T_DX_ADDR: u32 = 35;
pub const REG_T_DY_ADDR: u32 = 36;

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("ColorThrust");

    m.output("reg_bus_ready", m.high());
    let reg_bus_enable = m.input("reg_bus_enable", 1);
    let reg_bus_addr = m.input("reg_bus_addr", REG_BUS_ADDR_BIT_WIDTH);
    let reg_bus_write = m.input("reg_bus_write", 1);
    let reg_bus_write_data = m.input("reg_bus_write_data", 32);

    let reg_bus_write_enable = reg_bus_enable & reg_bus_write;

    let reg_depth_settings = m.reg("depth_settings", REG_DEPTH_SETTINGS_BITS);
    reg_depth_settings.drive_next(if_(reg_bus_write_enable & reg_bus_addr.eq(m.lit(REG_DEPTH_SETTINGS_ADDR, REG_BUS_ADDR_BIT_WIDTH)), {
        reg_bus_write_data.bits(REG_DEPTH_SETTINGS_BITS - 1, 0)
    }).else_({
        reg_depth_settings.value
    }));
    let depth_test_enable = reg_depth_settings.value.bit(REG_DEPTH_TEST_ENABLE_BIT);
    let depth_write_mask_enable = reg_depth_settings.value.bit(REG_DEPTH_WRITE_MASK_ENABLE_BIT);

    let reg_texture_settings = m.reg("texture_settings", REG_TEXTURE_SETTINGS_BITS);
    reg_texture_settings.drive_next(if_(reg_bus_write_enable & reg_bus_addr.eq(m.lit(REG_TEXTURE_SETTINGS_ADDR, REG_BUS_ADDR_BIT_WIDTH)), {
        reg_bus_write_data.bits(REG_TEXTURE_SETTINGS_BITS - 1, 0)
    }).else_({
        reg_texture_settings.value
    }));
    let tex_filter_select = reg_texture_settings.value.bit(REG_TEXTURE_SETTINGS_FILTER_SELECT_BIT);

    let reg_blend_settings = m.reg("blend_settings", REG_BLEND_SETTINGS_BITS);
    reg_blend_settings.drive_next(if_(reg_bus_write_enable & reg_bus_addr.eq(m.lit(REG_BLEND_SETTINGS_ADDR, REG_BUS_ADDR_BIT_WIDTH)), {
        reg_bus_write_data.bits(REG_BLEND_SETTINGS_BITS - 1, 0)
    }).else_({
        reg_blend_settings.value
    }));
    let blend_src_factor = reg_blend_settings.value.bits(REG_BLEND_SETTINGS_SRC_FACTOR_BIT_OFFSET + REG_BLEND_SETTINGS_SRC_FACTOR_BITS - 1, REG_BLEND_SETTINGS_SRC_FACTOR_BIT_OFFSET);
    let blend_dst_factor = reg_blend_settings.value.bits(REG_BLEND_SETTINGS_DST_FACTOR_BIT_OFFSET + REG_BLEND_SETTINGS_DST_FACTOR_BITS - 1, REG_BLEND_SETTINGS_DST_FACTOR_BIT_OFFSET);

    let input_generator_active = m.reg("input_generator_active", 1);
    input_generator_active.default_value(false);

    let tile_x = m.reg("tile_x", TILE_DIM_BITS);
    let tile_y = m.reg("tile_y", TILE_DIM_BITS);
    let tile_x_last = tile_x.value.eq(m.lit(TILE_DIM - 1, TILE_DIM_BITS));
    let tile_y_last = tile_y.value.eq(m.lit(TILE_DIM - 1, TILE_DIM_BITS));

    let start = reg_bus_write_enable & reg_bus_addr.eq(m.lit(REG_START_ADDR, REG_BUS_ADDR_BIT_WIDTH));

    let (next_input_generator_active, next_tile_x, next_tile_y) = if_(start, {
        let next_input_generator_active = m.high();

        let next_tile_x = m.lit(0u32, TILE_DIM_BITS);
        let next_tile_y = m.lit(0u32, TILE_DIM_BITS);

        (next_input_generator_active, next_tile_x, next_tile_y)
    }).else_({
        let next_input_generator_active = input_generator_active.value;

        let next_tile_x = tile_x.value + m.lit(1u32, TILE_DIM_BITS);
        let next_tile_y = tile_y.value;

        let (next_input_generator_active, next_tile_y) = if_(tile_x_last, {
            let next_input_generator_active = if_(tile_y_last, {
                m.low()
            }).else_({
                next_input_generator_active
            });

            let next_tile_y = tile_y.value + m.lit(1u32, TILE_DIM_BITS);

            (next_input_generator_active, next_tile_y)
        }).else_({
            (next_input_generator_active, next_tile_y)
        });

        (next_input_generator_active, next_tile_x, next_tile_y)
    });

    input_generator_active.drive_next(next_input_generator_active);

    tile_x.drive_next(next_tile_x);
    tile_y.drive_next(next_tile_y);

    let interpolant = |name: &str, num_bits, min_addr: u32, dx_addr: u32, dy_addr: u32| {
        let min = m.reg(format!("{}_min", name), num_bits);
        min.drive_next(if_(reg_bus_write_enable & reg_bus_addr.eq(m.lit(min_addr, REG_BUS_ADDR_BIT_WIDTH)), {
            reg_bus_write_data.bits(num_bits - 1, 0)
        }).else_({
            min.value
        }));
        let dx = m.reg(format!("{}_dx", name), num_bits);
        dx.drive_next(if_(reg_bus_write_enable & reg_bus_addr.eq(m.lit(dx_addr, REG_BUS_ADDR_BIT_WIDTH)), {
            reg_bus_write_data.bits(num_bits - 1, 0)
        }).else_({
            dx.value
        }));
        let dx_mirror = m.reg(format!("{}_dx_mirror", name), num_bits);
        dx_mirror.drive_next(if_(start, {
            dx.value
        }).else_({
            dx_mirror.value
        }));
        let dy = m.reg(format!("{}_dy", name), num_bits);
        dy.drive_next(if_(reg_bus_write_enable & reg_bus_addr.eq(m.lit(dy_addr, REG_BUS_ADDR_BIT_WIDTH)), {
            reg_bus_write_data.bits(num_bits - 1, 0)
        }).else_({
            dy.value
        }));
        let dy_mirror = m.reg(format!("{}_dy_mirror", name), num_bits);
        dy_mirror.drive_next(if_(start, {
            dy.value
        }).else_({
            dy_mirror.value
        }));

        let row = m.reg(format!("{}_row", name), num_bits);

        let value = m.reg(name, num_bits);

        let (next_row, next_value) = if_(start, {
            (min.value, min.value)
        }).else_({
            if_(tile_x_last, {
                let next = row.value + dy_mirror.value;
                (next, next)
            }).else_({
                (row.value, value.value + dx_mirror.value)
            })
        });

        row.drive_next(next_row);

        value.drive_next(next_value);

        value.value
    };

    let w0 = interpolant("w0", 32, REG_W0_MIN_ADDR, REG_W0_DX_ADDR, REG_W0_DY_ADDR).bit(31);
    let w1 = interpolant("w1", 32, REG_W1_MIN_ADDR, REG_W1_DX_ADDR, REG_W1_DY_ADDR).bit(31);
    let w2 = interpolant("w2", 32, REG_W2_MIN_ADDR, REG_W2_DX_ADDR, REG_W2_DY_ADDR).bit(31);

    let r = interpolant("r", 24, REG_R_MIN_ADDR, REG_R_DX_ADDR, REG_R_DY_ADDR).bits(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1, COLOR_FRACT_BITS);
    let g = interpolant("g", 24, REG_G_MIN_ADDR, REG_G_DX_ADDR, REG_G_DY_ADDR).bits(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1, COLOR_FRACT_BITS);
    let b = interpolant("b", 24, REG_B_MIN_ADDR, REG_B_DX_ADDR, REG_B_DY_ADDR).bits(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1, COLOR_FRACT_BITS);
    let a = interpolant("a", 24, REG_A_MIN_ADDR, REG_A_DX_ADDR, REG_A_DY_ADDR).bits(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1, COLOR_FRACT_BITS);

    let w_inverse = interpolant("w_inverse", 32, REG_W_INVERSE_MIN_ADDR, REG_W_INVERSE_DX_ADDR, REG_W_INVERSE_DY_ADDR);

    let z = interpolant("z", 32, REG_Z_MIN_ADDR, REG_Z_DX_ADDR, REG_Z_DY_ADDR).bits(31, 16);

    let s = interpolant("s", 32, REG_S_MIN_ADDR, REG_S_DX_ADDR, REG_S_DY_ADDR).bits(31, RESTORED_W_FRACT_BITS);
    let t = interpolant("t", 32, REG_T_MIN_ADDR, REG_T_DX_ADDR, REG_T_DY_ADDR).bits(31, RESTORED_W_FRACT_BITS);

    generate_pixel_pipe(c);
    let pixel_pipe = m.instance("pixel_pipe", "PixelPipe");

    pixel_pipe.drive_input("start", start);

    pixel_pipe.drive_input("depth_test_enable", depth_test_enable);
    pixel_pipe.drive_input("depth_write_mask_enable", depth_write_mask_enable);

    pixel_pipe.drive_input("tex_filter_select", tex_filter_select);

    pixel_pipe.drive_input("blend_src_factor", blend_src_factor);
    pixel_pipe.drive_input("blend_dst_factor", blend_dst_factor);

    pixel_pipe.drive_input("in_valid", input_generator_active.value);
    pixel_pipe.drive_input("in_tile_addr", tile_y.value.concat(tile_x.value));

    pixel_pipe.drive_input("in_w0", w0);
    pixel_pipe.drive_input("in_w1", w1);
    pixel_pipe.drive_input("in_w2", w2);

    pixel_pipe.drive_input("in_r", r);
    pixel_pipe.drive_input("in_g", g);
    pixel_pipe.drive_input("in_b", b);
    pixel_pipe.drive_input("in_a", a);

    pixel_pipe.drive_input("in_w_inverse", w_inverse);

    pixel_pipe.drive_input("in_z", z);

    pixel_pipe.drive_input("in_s", s);
    pixel_pipe.drive_input("in_t", t);

    m.output("reg_bus_read_data", m.lit(0u32, 31).concat(input_generator_active.value | pixel_pipe.output("active")));
    m.output("reg_bus_read_data_valid", (reg_bus_enable & !reg_bus_write).reg_next_with_default("reg_bus_read_data_valid", false));

    m.output("color_buffer_bus_ready", m.high());
    let color_buffer_bus_enable = m.input("color_buffer_bus_enable", 1);
    let color_buffer_bus_addr = m.input("color_buffer_bus_addr", TILE_PIXELS_WORDS_BITS);
    let color_buffer_bus_write = m.input("color_buffer_bus_write", 1);
    let color_buffer_bus_write_data = m.input("color_buffer_bus_write_data", 128);
    let color_buffer_bus_write_byte_enable = m.input("color_buffer_bus_write_byte_enable", 16);
    let color_buffer_bus_write_word_enable = (0..4).fold(None, |acc, x| {
        let word_enable_bit = color_buffer_bus_write_byte_enable.bit(x * 4);
        Some(if let Some(acc) = acc {
            word_enable_bit.concat(acc)
        } else {
            word_enable_bit
        })
    }).unwrap();

    let color_buffer = WordMem::new(m, "color_buffer", TILE_PIXELS_WORDS_BITS, 32, 4);
    let color_buffer_bus_write_enable = color_buffer_bus_enable & color_buffer_bus_write;
    color_buffer.write_port(
        if_(color_buffer_bus_write_enable, {
            color_buffer_bus_addr
        }).else_({
            pixel_pipe.output("color_buffer_write_port_addr")
        }),
        if_(color_buffer_bus_write_enable, {
            color_buffer_bus_write_data
        }).else_({
            pixel_pipe.output("color_buffer_write_port_value")
        }),
        color_buffer_bus_write_enable | pixel_pipe.output("color_buffer_write_port_enable"),
        if_(color_buffer_bus_write_enable, {
            color_buffer_bus_write_word_enable
        }).else_({
            pixel_pipe.output("color_buffer_write_port_word_enable")
        }));

    let color_buffer_bus_read_enable = color_buffer_bus_enable & !color_buffer_bus_write;
    let color_buffer_read_port_value = color_buffer.read_port(
        if_(color_buffer_bus_read_enable, {
            color_buffer_bus_addr
        }).else_({
            pixel_pipe.output("color_buffer_read_port_addr")
        }),
        color_buffer_bus_read_enable | pixel_pipe.output("color_buffer_read_port_enable"));

    pixel_pipe.drive_input("color_buffer_read_port_value", color_buffer_read_port_value);

    m.output("color_buffer_bus_read_data", color_buffer_read_port_value);
    m.output("color_buffer_bus_read_data_valid", color_buffer_bus_read_enable.reg_next_with_default("color_buffer_bus_read_data_valid", false));

    m.output("depth_buffer_bus_ready", m.high());
    let depth_buffer_bus_enable = m.input("depth_buffer_bus_enable", 1);
    let depth_buffer_bus_addr = m.input("depth_buffer_bus_addr", TILE_PIXELS_WORDS_BITS - 1);
    let depth_buffer_bus_write = m.input("depth_buffer_bus_write", 1);
    let depth_buffer_bus_write_data = m.input("depth_buffer_bus_write_data", 128);
    let depth_buffer_bus_write_byte_enable = m.input("depth_buffer_bus_write_byte_enable", 16);
    let depth_buffer_bus_write_word_enable = (0..8).fold(None, |acc, x| {
        let word_enable_bit = depth_buffer_bus_write_byte_enable.bit(x * 2);
        Some(if let Some(acc) = acc {
            word_enable_bit.concat(acc)
        } else {
            word_enable_bit
        })
    }).unwrap();

    let depth_buffer = WordMem::new(m, "depth_buffer", TILE_PIXELS_WORDS_BITS - 1, 16, 8);
    let depth_buffer_bus_write_enable = depth_buffer_bus_enable & depth_buffer_bus_write;
    depth_buffer.write_port(
        if_(depth_buffer_bus_write_enable, {
            depth_buffer_bus_addr
        }).else_({
            pixel_pipe.output("depth_buffer_write_port_addr")
        }),
        if_(depth_buffer_bus_write_enable, {
            depth_buffer_bus_write_data
        }).else_({
            pixel_pipe.output("depth_buffer_write_port_value")
        }),
        depth_buffer_bus_write_enable | pixel_pipe.output("depth_buffer_write_port_enable"),
        if_(depth_buffer_bus_write_enable, {
            depth_buffer_bus_write_word_enable
        }).else_({
            pixel_pipe.output("depth_buffer_write_port_word_enable")
        }));

    let depth_buffer_bus_read_enable = depth_buffer_bus_enable & !depth_buffer_bus_write;
    let depth_buffer_read_port_value = depth_buffer.read_port(
        if_(depth_buffer_bus_read_enable, {
            depth_buffer_bus_addr
        }).else_({
            pixel_pipe.output("depth_buffer_read_port_addr")
        }),
        depth_buffer_bus_read_enable | pixel_pipe.output("depth_buffer_read_port_enable"));

    pixel_pipe.drive_input("depth_buffer_read_port_value", depth_buffer_read_port_value);

    m.output("depth_buffer_bus_read_data", depth_buffer_read_port_value);
    m.output("depth_buffer_bus_read_data_valid", depth_buffer_bus_read_enable.reg_next_with_default("depth_buffer_bus_read_data_valid", false));

    m.output("tex_buffer_bus_ready", m.high());
    let tex_buffer_bus_enable = m.input("tex_buffer_bus_enable", 1);
    let tex_buffer_bus_addr = m.input("tex_buffer_bus_addr", TEX_PIXELS_WORDS_BITS);
    let tex_buffer_bus_write_data = m.input("tex_buffer_bus_write_data", 128);
    let tex_buffer_bus_write_byte_enable = m.input("tex_buffer_bus_write_byte_enable", 16);

    for i in 0..4 {
        let tex_buffer = m.mem(format!("tex_buffer{}", i), TEX_BUFFER_PIXELS_BITS, 32);
        let tex_buffer_bus_write_data_offset = i * 32;
        tex_buffer.write_port(
            tex_buffer_bus_addr,
            tex_buffer_bus_write_data.bits(tex_buffer_bus_write_data_offset + 31, tex_buffer_bus_write_data_offset),
            tex_buffer_bus_enable & tex_buffer_bus_write_byte_enable.bit(i * 4));

        let read_port_value = tex_buffer.read_port(
            pixel_pipe.output(format!("tex_buffer{}_read_port_addr", i)),
            pixel_pipe.output(format!("tex_buffer{}_read_port_enable", i)));

        pixel_pipe.drive_input(format!("tex_buffer{}_read_port_value", i), read_port_value);
    }

    m
}

pub fn generate_pixel_pipe<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("PixelPipe");

    // Control
    let start = m.input("start", 1);

    let active = m.reg("active", 1);
    active.default_value(false);
    m.output("active", active.value);

    let finished_pixel_acc = m.reg("finished_pixel_acc", TILE_PIXELS_BITS + 1);

    // Inputs
    let valid = m.input("in_valid", 1);
    let tile_addr = m.input("in_tile_addr", TILE_PIXELS_BITS);

    //  Reject pixel if it doesn't pass edge test before it enters the rest of the pipe
    let w0 = m.input("in_w0", 1);
    let w1 = m.input("in_w1", 1);
    let w2 = m.input("in_w2", 1);
    let edge_test = !(w0 | w1 | w2);
    let reject = valid & !edge_test;
    let valid = valid & edge_test;

    let r = m.input("in_r", COLOR_WHOLE_BITS);
    let g = m.input("in_g", COLOR_WHOLE_BITS);
    let b = m.input("in_b", COLOR_WHOLE_BITS);
    let a = m.input("in_a", COLOR_WHOLE_BITS);

    let w_inverse = m.input("in_w_inverse", 32);

    let z = m.input("in_z", 16);

    let s = m.input("in_s", 32 - RESTORED_W_FRACT_BITS);
    let t = m.input("in_t", 32 - RESTORED_W_FRACT_BITS);

    // Depth test pipe
    generate_depth_test_pipe(c);
    let depth_test_pipe = m.instance("depth_test_pipe", "DepthTestPipe");

    //  Aux
    depth_test_pipe.drive_input("depth_test_enable", m.input("depth_test_enable", 1));

    m.output("depth_buffer_read_port_addr", depth_test_pipe.output("depth_buffer_read_port_addr"));
    m.output("depth_buffer_read_port_enable", depth_test_pipe.output("depth_buffer_read_port_enable"));

    depth_test_pipe.drive_input("depth_buffer_read_port_value", m.input("depth_buffer_read_port_value", 128));

    //  Inputs
    depth_test_pipe.drive_input("in_valid", valid);
    depth_test_pipe.drive_input("in_tile_addr", tile_addr);

    depth_test_pipe.drive_input("in_r", r);
    depth_test_pipe.drive_input("in_g", g);
    depth_test_pipe.drive_input("in_b", b);
    depth_test_pipe.drive_input("in_a", a);

    depth_test_pipe.drive_input("in_w_inverse", w_inverse);

    depth_test_pipe.drive_input("in_z", z);

    depth_test_pipe.drive_input("in_s", s);
    depth_test_pipe.drive_input("in_t", t);

    //  Outputs
    let valid = depth_test_pipe.output("out_valid");
    let tile_addr = depth_test_pipe.output("out_tile_addr");

    let r = depth_test_pipe.output("out_r");
    let g = depth_test_pipe.output("out_g");
    let b = depth_test_pipe.output("out_b");
    let a = depth_test_pipe.output("out_a");

    let w_inverse = depth_test_pipe.output("out_w_inverse");

    let z = depth_test_pipe.output("out_z");

    let s = depth_test_pipe.output("out_s");
    let t = depth_test_pipe.output("out_t");

    let depth_test_result = depth_test_pipe.output("out_depth_test_result");

    // Back pipe
    generate_back_pipe(c);
    let back_pipe = m.instance("back_pipe", "BackPipe");

    //  Aux
    back_pipe.drive_input("depth_write_mask_enable", m.input("depth_write_mask_enable", 1));

    back_pipe.drive_input("tex_filter_select", m.input("tex_filter_select", 1));

    back_pipe.drive_input("blend_src_factor", m.input("blend_src_factor", REG_BLEND_SETTINGS_SRC_FACTOR_BITS));
    back_pipe.drive_input("blend_dst_factor", m.input("blend_dst_factor", REG_BLEND_SETTINGS_DST_FACTOR_BITS));

    for i in 0..4 {
        m.output(format!("tex_buffer{}_read_port_addr", i), back_pipe.output(format!("tex_buffer{}_read_port_addr", i)));
        m.output(format!("tex_buffer{}_read_port_enable", i), back_pipe.output(format!("tex_buffer{}_read_port_enable", i)));

        back_pipe.drive_input(format!("tex_buffer{}_read_port_value", i), m.input(format!("tex_buffer{}_read_port_value", i), 32));
    }

    m.output("color_buffer_read_port_addr", back_pipe.output("color_buffer_read_port_addr"));
    m.output("color_buffer_read_port_enable", back_pipe.output("color_buffer_read_port_enable"));

    back_pipe.drive_input("color_buffer_read_port_value", m.input("color_buffer_read_port_value", 128));

    m.output("color_buffer_write_port_addr", back_pipe.output("color_buffer_write_port_addr"));
    m.output("color_buffer_write_port_value", back_pipe.output("color_buffer_write_port_value"));
    m.output("color_buffer_write_port_enable", back_pipe.output("color_buffer_write_port_enable"));
    m.output("color_buffer_write_port_word_enable", back_pipe.output("color_buffer_write_port_word_enable"));

    m.output("depth_buffer_write_port_addr", back_pipe.output("depth_buffer_write_port_addr"));
    m.output("depth_buffer_write_port_value", back_pipe.output("depth_buffer_write_port_value"));
    m.output("depth_buffer_write_port_enable", back_pipe.output("depth_buffer_write_port_enable"));
    m.output("depth_buffer_write_port_word_enable", back_pipe.output("depth_buffer_write_port_word_enable"));

    //  Inputs
    back_pipe.drive_input("in_valid", valid);
    back_pipe.drive_input("in_tile_addr", tile_addr);

    back_pipe.drive_input("in_r", r);
    back_pipe.drive_input("in_g", g);
    back_pipe.drive_input("in_b", b);
    back_pipe.drive_input("in_a", a);

    back_pipe.drive_input("in_w_inverse", w_inverse);

    back_pipe.drive_input("in_z", z);

    back_pipe.drive_input("in_s", s);
    back_pipe.drive_input("in_t", t);

    back_pipe.drive_input("in_depth_test_result", depth_test_result);

    //  Outputs
    let valid = back_pipe.output("out_valid");

    active.drive_next(if_(start, {
        m.high()
    }).else_if(finished_pixel_acc.value.eq(m.lit(TILE_PIXELS, TILE_PIXELS_BITS + 1)), {
        m.low()
    }).else_({
        active.value
    }));

    // Fancy decode to map disjoint, binary finished/reject signals to a count
    //  00 -> 00
    //  01 -> 01
    //  10 -> 01
    //  11 -> 10
    let finished_pixel_count = (valid & reject).concat(valid ^ reject);

    finished_pixel_acc.drive_next(if_(start, {
        m.lit(0u32, TILE_PIXELS_BITS + 1)
    }).else_({
        finished_pixel_acc.value + m.lit(0u32, TILE_PIXELS_BITS - 1).concat(finished_pixel_count)
    }));

    m
}

pub fn generate_depth_test_pipe<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("DepthTestPipe");

    // Aux inputs
    let depth_test_enable = m.input("depth_test_enable", 1);

    // Inputs
    let valid = m.input("in_valid", 1);
    let tile_addr = m.input("in_tile_addr", TILE_PIXELS_BITS);

    let r = m.input("in_r", COLOR_WHOLE_BITS);
    let g = m.input("in_g", COLOR_WHOLE_BITS);
    let b = m.input("in_b", COLOR_WHOLE_BITS);
    let a = m.input("in_a", COLOR_WHOLE_BITS);

    let w_inverse = m.input("in_w_inverse", 32);

    let z = m.input("in_z", 16);

    let s = m.input("in_s", 32 - RESTORED_W_FRACT_BITS);
    let t = m.input("in_t", 32 - RESTORED_W_FRACT_BITS);

    //  Issue depth buffer read for prev_depth
    m.output("depth_buffer_read_port_addr", tile_addr.bits(TILE_PIXELS_BITS - 1, 3));
    m.output("depth_buffer_read_port_enable", valid & depth_test_enable);

    // Stage 1
    let valid = valid.reg_next_with_default("stage_1_valid", false);
    let tile_addr = tile_addr.reg_next("stage_1_tile_addr");

    let r = r.reg_next("stage_1_r");
    let g = g.reg_next("stage_1_g");
    let b = b.reg_next("stage_1_b");
    let a = a.reg_next("stage_1_a");

    let w_inverse = w_inverse.reg_next("stage_1_w_inverse");

    let z = z.reg_next("stage_1_z");

    let s = s.reg_next("stage_1_s");
    let t = t.reg_next("stage_1_t");

    //  Returned from issue in previous stage
    let prev_depth = m.input("depth_buffer_read_port_value", 128);
    let prev_depth = if_(tile_addr.bits(2, 0).eq(m.lit(0u32, 3)), {
        prev_depth.bits(15, 0)
    }).else_if(tile_addr.bits(2, 0).eq(m.lit(1u32, 3)), {
        prev_depth.bits(31, 16)
    }).else_if(tile_addr.bits(2, 0).eq(m.lit(2u32, 3)), {
        prev_depth.bits(47, 32)
    }).else_if(tile_addr.bits(2, 0).eq(m.lit(3u32, 3)), {
        prev_depth.bits(63, 48)
    }).else_if(tile_addr.bits(2, 0).eq(m.lit(4u32, 3)), {
        prev_depth.bits(79, 64)
    }).else_if(tile_addr.bits(2, 0).eq(m.lit(5u32, 3)), {
        prev_depth.bits(95, 80)
    }).else_if(tile_addr.bits(2, 0).eq(m.lit(6u32, 3)), {
        prev_depth.bits(111, 96)
    }).else_({
        prev_depth.bits(127, 112)
    });

    // Stage 2
    let valid = valid.reg_next_with_default("stage_2_valid", false);
    let tile_addr = tile_addr.reg_next("stage_2_tile_addr");

    let r = r.reg_next("stage_2_r");
    let g = g.reg_next("stage_2_g");
    let b = b.reg_next("stage_2_b");
    let a = a.reg_next("stage_2_a");

    let w_inverse = w_inverse.reg_next("stage_2_w_inverse");

    let z = z.reg_next("stage_2_z");

    let s = s.reg_next("stage_2_s");
    let t = t.reg_next("stage_2_t");

    let prev_depth = prev_depth.reg_next("stage_2_prev_depth");

    let depth_test_result = z.lt(prev_depth) | !depth_test_enable;

    // Outputs
    m.output("out_valid", valid);
    m.output("out_tile_addr", tile_addr);

    m.output("out_r", r);
    m.output("out_g", g);
    m.output("out_b", b);
    m.output("out_a", a);

    m.output("out_w_inverse", w_inverse);

    m.output("out_z", z);

    m.output("out_s", s);
    m.output("out_t", t);

    m.output("out_depth_test_result", depth_test_result);

    m
}

pub fn generate_back_pipe<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("BackPipe");

    // Aux inputs
    let depth_write_mask_enable = m.input("depth_write_mask_enable", 1);

    let tex_filter_select = m.input("tex_filter_select", 1);

    let blend_src_factor = m.input("blend_src_factor", REG_BLEND_SETTINGS_SRC_FACTOR_BITS);
    let blend_dst_factor = m.input("blend_dst_factor", REG_BLEND_SETTINGS_DST_FACTOR_BITS);

    // Inputs
    let mut valid = m.input("in_valid", 1);
    let mut tile_addr = m.input("in_tile_addr", TILE_PIXELS_BITS);

    let mut r = m.input("in_r", COLOR_WHOLE_BITS);
    let mut g = m.input("in_g", COLOR_WHOLE_BITS);
    let mut b = m.input("in_b", COLOR_WHOLE_BITS);
    let mut a = m.input("in_a", COLOR_WHOLE_BITS);

    let w_inverse = m.input("in_w_inverse", 32);

    let mut z = m.input("in_z", 16);

    let mut s = m.input("in_s", 32 - RESTORED_W_FRACT_BITS);
    let mut t = m.input("in_t", 32 - RESTORED_W_FRACT_BITS);

    let mut depth_test_result = m.input("in_depth_test_result", 1);

    approx_reciprocal::generate(c, "WInverseReciprocal", W_INVERSE_FRACT_BITS - RESTORED_W_FRACT_BITS - 3, 4);
    let w_approx_reciprocal = m.instance("w_approx_reciprocal", "WInverseReciprocal");
    w_approx_reciprocal.drive_input("x", w_inverse);

    // Stages 1-13 (mostly just delay for w to arrive)
    for stage in 1..=13 {
        valid = valid.reg_next_with_default(format!("stage_{}_valid", stage), false);
        tile_addr = tile_addr.reg_next(format!("stage_{}_tile_addr", stage));

        r = r.reg_next(format!("stage_{}_r", stage));
        g = g.reg_next(format!("stage_{}_g", stage));
        b = b.reg_next(format!("stage_{}_b", stage));
        a = a.reg_next(format!("stage_{}_a", stage));

        z = z.reg_next(format!("stage_{}_z", stage));

        s = s.reg_next(format!("stage_{}_s", stage));
        t = t.reg_next(format!("stage_{}_t", stage));

        depth_test_result = depth_test_result.reg_next(format!("stage_{}_depth_test_result", stage));
    }

    //  Returned from issue before stage 1
    let w = w_approx_reciprocal.output("quotient");

    // Stage 14
    let valid = valid.reg_next_with_default("stage_14_valid", false);
    let tile_addr = tile_addr.reg_next("stage_14_tile_addr");

    let r = r.reg_next("stage_14_r");
    let g = g.reg_next("stage_14_g");
    let b = b.reg_next("stage_14_b");
    let a = a.reg_next("stage_14_a");

    let z = z.reg_next("stage_14_z");

    let s = s.reg_next("stage_14_s");
    let t = t.reg_next("stage_14_t");

    let w = w.reg_next("stage_14_w");

    let s = s.mul_signed(w);
    let t = t.mul_signed(w);

    let depth_test_result = depth_test_result.reg_next("stage_14_depth_test_result");

    // Stage 15
    let valid = valid.reg_next_with_default("stage_15_valid", false);
    let tile_addr = tile_addr.reg_next("stage_15_tile_addr");

    let r = r.reg_next("stage_15_r");
    let g = g.reg_next("stage_15_g");
    let b = b.reg_next("stage_15_b");
    let a = a.reg_next("stage_15_a");

    let z = z.reg_next("stage_15_z");

    let s = s.reg_next("stage_15_s");
    let t = t.reg_next("stage_15_t");

    let depth_test_result = depth_test_result.reg_next("stage_15_depth_test_result");

    let s_floor = s.bits(31, ST_FRACT_BITS);
    let t_floor = t.bits(31, ST_FRACT_BITS);
    let s_fract = m.low().concat(s.bits(ST_FRACT_BITS - 1, ST_FRACT_BITS - ST_FILTER_FRACT_BITS));
    let t_fract = m.low().concat(t.bits(ST_FRACT_BITS - 1, ST_FRACT_BITS - ST_FILTER_FRACT_BITS));
    let one_minus_s_fract = m.high().concat(m.lit(0u32, ST_FILTER_FRACT_BITS)) - s_fract;
    let one_minus_t_fract = m.high().concat(m.lit(0u32, ST_FILTER_FRACT_BITS)) - t_fract;

    //  Lock weights for nearest filtering
    let (s_fract, one_minus_s_fract, t_fract, one_minus_t_fract) = if_(!tex_filter_select, {
        let zero = m.low().concat(m.lit(0u32, ST_FILTER_FRACT_BITS));
        let one = m.high().concat(m.lit(0u32, ST_FILTER_FRACT_BITS));
        (zero, one, zero, one)
    }).else_({
        (s_fract, one_minus_s_fract, t_fract, one_minus_t_fract)
    });

    //  Swap weights depending on pixel offsets
    let (s_fract, one_minus_s_fract) = if_(!s_floor.bit(0), {
        (s_fract, one_minus_s_fract)
    }).else_({
        (one_minus_s_fract, s_fract)
    });
    let (t_fract, one_minus_t_fract) = if_(!t_floor.bit(0), {
        (t_fract, one_minus_t_fract)
    }).else_({
        (one_minus_t_fract, t_fract)
    });

    //  Issue tex buffer reads for filtered texel
    let buffer0_s = (s_floor.bits(3, 0) + m.lit(1u32, 4)).bits(3, 1);
    let buffer0_t = (t_floor.bits(3, 0) + m.lit(1u32, 4)).bits(3, 1);
    let buffer1_s = s_floor.bits(3, 1);
    let buffer1_t = (t_floor.bits(3, 0) + m.lit(1u32, 4)).bits(3, 1);
    let buffer2_s = (s_floor.bits(3, 0) + m.lit(1u32, 4)).bits(3, 1);
    let buffer2_t = t_floor.bits(3, 1);
    let buffer3_s = s_floor.bits(3, 1);
    let buffer3_t = t_floor.bits(3, 1);
    m.output("tex_buffer0_read_port_addr", buffer0_t.concat(buffer0_s));
    m.output("tex_buffer1_read_port_addr", buffer1_t.concat(buffer1_s));
    m.output("tex_buffer2_read_port_addr", buffer2_t.concat(buffer2_s));
    m.output("tex_buffer3_read_port_addr", buffer3_t.concat(buffer3_s));
    for i in 0..4 {
        m.output(format!("tex_buffer{}_read_port_enable", i), valid);
    }

    // Stage 16
    let valid = valid.reg_next_with_default("stage_16_valid", false);
    let tile_addr = tile_addr.reg_next("stage_16_tile_addr");

    let r = r.reg_next("stage_16_r");
    let g = g.reg_next("stage_16_g");
    let b = b.reg_next("stage_16_b");
    let a = a.reg_next("stage_16_a");

    let z = z.reg_next("stage_16_z");

    let depth_test_result = depth_test_result.reg_next("stage_16_depth_test_result");

    let s_fract = s_fract.reg_next("stage_16_s_fract");
    let t_fract = t_fract.reg_next("stage_16_t_fract");
    let one_minus_s_fract = one_minus_s_fract.reg_next("stage_16_one_minus_s_fract");
    let one_minus_t_fract = one_minus_t_fract.reg_next("stage_16_one_minus_t_fract");

    struct Texel<'a> {
        r: &'a Signal<'a>,
        g: &'a Signal<'a>,
        b: &'a Signal<'a>,
        a: &'a Signal<'a>,
    }

    impl<'a> Texel<'a> {
        fn new(texel: &'a Signal<'a>) -> Texel<'a> {
            Texel {
                r: texel.bits(23, 16),
                g: texel.bits(15, 8),
                b: texel.bits(7, 0),
                a: texel.bits(31, 24),
            }
        }

        fn argb(&self) -> &'a Signal<'a> {
            self.a.concat(self.r).concat(self.g).concat(self.b)
        }
    }

    fn blend_component<'a>(a: &'a Signal<'a>, b: &'a Signal<'a>, a_fract: &'a Signal<'a>, b_fract: &'a Signal<'a>) -> &'a Signal<'a> {
        (a * a_fract + b * b_fract).bits(8 + ST_FILTER_FRACT_BITS - 1, ST_FILTER_FRACT_BITS)
    }

    fn blend_texels<'a>(a: &Texel<'a>, b: &Texel<'a>, a_fract: &'a Signal<'a>, b_fract: &'a Signal<'a>) -> Texel<'a> {
        Texel {
            r: blend_component(a.r, b.r, a_fract, b_fract),
            g: blend_component(a.g, b.g, a_fract, b_fract),
            b: blend_component(a.b, b.b, a_fract, b_fract),
            a: blend_component(a.a, b.a, a_fract, b_fract),
        }
    }

    //  Returned from issues in previous stage
    let texel0 = Texel::new(m.input("tex_buffer0_read_port_value", 32));
    let texel1 = Texel::new(m.input("tex_buffer1_read_port_value", 32));
    let texel2 = Texel::new(m.input("tex_buffer2_read_port_value", 32));
    let texel3 = Texel::new(m.input("tex_buffer3_read_port_value", 32));

    let lower = blend_texels(&texel0, &texel1, one_minus_s_fract, s_fract).argb();
    let upper = blend_texels(&texel2, &texel3, one_minus_s_fract, s_fract).argb();

    // Stage 17
    let valid = valid.reg_next_with_default("stage_17_valid", false);
    let tile_addr = tile_addr.reg_next("stage_17_tile_addr");

    let r = r.reg_next("stage_17_r");
    let g = g.reg_next("stage_17_g");
    let b = b.reg_next("stage_17_b");
    let a = a.reg_next("stage_17_a");

    let z = z.reg_next("stage_17_z");

    let depth_test_result = depth_test_result.reg_next("stage_17_depth_test_result");

    let t_fract = t_fract.reg_next("stage_17_t_fract");
    let one_minus_t_fract = one_minus_t_fract.reg_next("stage_17_one_minus_t_fract");

    let lower = Texel::new(lower.reg_next("stage_17_lower"));
    let upper = Texel::new(upper.reg_next("stage_17_upper"));

    let texel = blend_texels(&lower, &upper, one_minus_t_fract, t_fract).argb();

    // Stage 18
    let valid = valid.reg_next_with_default("stage_18_valid", false);
    let tile_addr = tile_addr.reg_next("stage_18_tile_addr");

    let r = r.reg_next("stage_18_r");
    let g = g.reg_next("stage_18_g");
    let b = b.reg_next("stage_18_b");
    let a = a.reg_next("stage_18_a");

    let z = z.reg_next("stage_18_z");

    let depth_test_result = depth_test_result.reg_next("stage_18_depth_test_result");

    let texel = texel.reg_next("stage_18_texel");

    let scale_comp = |color_comp: &'a Signal<'a>, texel_comp: &'a Signal<'a>| -> &'a Signal<'a> {
        (color_comp * texel_comp).bits(16, 8)
    };

    let r = scale_comp(r, texel.bits(23, 16));
    let g = scale_comp(g, texel.bits(15, 8));
    let b = scale_comp(b, texel.bits(7, 0));
    let a = scale_comp(a, texel.bits(31, 24));

    //  Issue color buffer read for prev_color
    m.output("color_buffer_read_port_addr", tile_addr.bits(TILE_PIXELS_BITS - 1, 2));
    m.output("color_buffer_read_port_enable", valid);

    // Stage 19
    let valid = valid.reg_next_with_default("stage_19_valid", false);
    let tile_addr = tile_addr.reg_next("stage_19_tile_addr");

    let r = r.reg_next("stage_19_r");
    let g = g.reg_next("stage_19_g");
    let b = b.reg_next("stage_19_b");
    let a = a.reg_next("stage_19_a");

    let z = z.reg_next("stage_19_z");

    let depth_test_result = depth_test_result.reg_next("stage_19_depth_test_result");

    let zero = m.lit(0u32, 9);
    let one = m.high().concat(m.lit(0u32, 8));

    let blend_src_factor = if_(blend_src_factor.eq(m.lit(REG_BLEND_SETTINGS_SRC_FACTOR_ZERO, REG_BLEND_SETTINGS_SRC_FACTOR_BITS)), {
        zero
    }).else_if(blend_src_factor.eq(m.lit(REG_BLEND_SETTINGS_SRC_FACTOR_ONE, REG_BLEND_SETTINGS_SRC_FACTOR_BITS)), {
        one
    }).else_if(blend_src_factor.eq(m.lit(REG_BLEND_SETTINGS_SRC_FACTOR_SRC_ALPHA, REG_BLEND_SETTINGS_SRC_FACTOR_BITS)), {
        a
    }).else_({
        one - a
    });

    let blend_dst_factor = if_(blend_dst_factor.eq(m.lit(REG_BLEND_SETTINGS_DST_FACTOR_ZERO, REG_BLEND_SETTINGS_DST_FACTOR_BITS)), {
        zero
    }).else_if(blend_dst_factor.eq(m.lit(REG_BLEND_SETTINGS_DST_FACTOR_ONE, REG_BLEND_SETTINGS_DST_FACTOR_BITS)), {
        one
    }).else_if(blend_dst_factor.eq(m.lit(REG_BLEND_SETTINGS_DST_FACTOR_SRC_ALPHA, REG_BLEND_SETTINGS_DST_FACTOR_BITS)), {
        a
    }).else_({
        one - a
    });

    //  Returned from issue in previous stage
    let prev_color = m.input("color_buffer_read_port_value", 128);
    let prev_color = if_(tile_addr.bits(1, 0).eq(m.lit(0u32, 2)), {
        prev_color.bits(31, 0)
    }).else_if(tile_addr.bits(1, 0).eq(m.lit(1u32, 2)), {
        prev_color.bits(63, 32)
    }).else_if(tile_addr.bits(1, 0).eq(m.lit(2u32, 2)), {
        prev_color.bits(95, 64)
    }).else_({
        prev_color.bits(127, 96)
    });

    // Stage 20
    let valid = valid.reg_next_with_default("stage_20_valid", false);
    let tile_addr = tile_addr.reg_next("stage_20_tile_addr");

    let r = r.reg_next("stage_20_r");
    let g = g.reg_next("stage_20_g");
    let b = b.reg_next("stage_20_b");
    let a = a.reg_next("stage_20_a");

    let z = z.reg_next("stage_20_z");

    let depth_test_result = depth_test_result.reg_next("stage_20_depth_test_result");

    let blend_src_factor = blend_src_factor.reg_next("stage_20_blend_src_factor");
    let blend_dst_factor = blend_dst_factor.reg_next("stage_20_blend_dst_factor");

    let prev_color = prev_color.reg_next("stage_20_prev_color");

    let r = (r * blend_src_factor).bits(17, 8);
    let g = (g * blend_src_factor).bits(17, 8);
    let b = (b * blend_src_factor).bits(17, 8);

    let prev_r = m.lit(0u32, 2).concat((prev_color.bits(23, 16) * blend_dst_factor).bits(16, 9));
    let prev_g = m.lit(0u32, 2).concat((prev_color.bits(15, 8) * blend_dst_factor).bits(16, 9));
    let prev_b = m.lit(0u32, 2).concat((prev_color.bits(7, 0) * blend_dst_factor).bits(16, 9));

    let clamp_comp = |comp: &'a Signal<'a>| -> &'a Signal<'a> {
        if_(comp.bits(9, 8).eq(m.lit(0u32, 2)), {
            comp.bits(7, 0)
        }).else_({
            m.lit(255u32, 8)
        })
    };

    let r = clamp_comp(r + prev_r);
    let g = clamp_comp(g + prev_g);
    let b = clamp_comp(b + prev_b);
    let a = clamp_comp(m.low().concat(a));

    let color = a.concat(r).concat(g).concat(b);

    // Stage 21
    let valid = valid.reg_next_with_default("stage_21_valid", false);
    let tile_addr = tile_addr.reg_next("stage_21_tile_addr");

    let z = z.reg_next("stage_21_z");

    let depth_test_result = depth_test_result.reg_next("stage_21_depth_test_result");

    let color = color.reg_next("stage_21_color");

    m.output("color_buffer_write_port_addr", tile_addr.bits(TILE_PIXELS_BITS - 1, 2));
    m.output("color_buffer_write_port_value", color.repeat(4));
    m.output("color_buffer_write_port_enable", valid & depth_test_result);
    m.output("color_buffer_write_port_word_enable", (0u32..4).fold(None, |acc, x| {
        let word_enable_bit = tile_addr.bits(1, 0).eq(m.lit(x, 2));
        Some(if let Some(acc) = acc {
            word_enable_bit.concat(acc)
        } else {
            word_enable_bit
        })
    }).unwrap());

    m.output("depth_buffer_write_port_addr", tile_addr.bits(TILE_PIXELS_BITS - 1, 3));
    m.output("depth_buffer_write_port_value", z.repeat(8));
    m.output("depth_buffer_write_port_enable", valid & depth_test_result & depth_write_mask_enable);
    m.output("depth_buffer_write_port_word_enable", (0u32..8).fold(None, |acc, x| {
        let word_enable_bit = tile_addr.bits(2, 0).eq(m.lit(x, 3));
        Some(if let Some(acc) = acc {
            word_enable_bit.concat(acc)
        } else {
            word_enable_bit
        })
    }).unwrap());

    m.output("out_valid", valid);

    m
}
