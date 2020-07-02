use crate::approx_reciprocal;
use crate::helpers::*;

use kaze::*;

pub const TILE_DIM_BITS: u32 = 4;
pub const TILE_DIM: u32 = 1 << TILE_DIM_BITS;
pub const TILE_PIXELS_BITS: u32 = TILE_DIM_BITS * 2;
pub const TILE_PIXELS: u32 = 1 << TILE_PIXELS_BITS;

pub const TEX_DIM_BITS: u32 = 4;
pub const TEX_DIM: u32 = 1 << TEX_DIM_BITS;
pub const TEX_PIXELS_BITS: u32 = TEX_DIM_BITS * 2;
pub const TEX_PIXELS: u32 = 1 << TEX_PIXELS_BITS;

pub const W_INVERSE_FRACT_BITS: u32 = 30;
pub const ST_FRACT_BITS: u32 = 24;
pub const ST_FILTER_BITS: u32 = 4; // Must be less than ST_FRACT_BITS
pub const RESTORED_W_FRACT_BITS: u32 = 8; // Must be less than W_INVERSE_FRACT_BITS and ST_FRACT_BITS

pub const REG_BUS_ADDR_BIT_WIDTH: u32 = 6;

pub const REG_STATUS_ADDR: u32 = 0;
pub const REG_START_ADDR: u32 = 0;

pub const REG_W0_MIN_ADDR: u32 = 1;
pub const REG_W0_DX_ADDR: u32 = 2;
pub const REG_W0_DY_ADDR: u32 = 3;
pub const REG_W1_MIN_ADDR: u32 = 4;
pub const REG_W1_DX_ADDR: u32 = 5;
pub const REG_W1_DY_ADDR: u32 = 6;
pub const REG_W2_MIN_ADDR: u32 = 7;
pub const REG_W2_DX_ADDR: u32 = 8;
pub const REG_W2_DY_ADDR: u32 = 9;
pub const REG_R_MIN_ADDR: u32 = 10;
pub const REG_R_DX_ADDR: u32 = 11;
pub const REG_R_DY_ADDR: u32 = 12;
pub const REG_G_MIN_ADDR: u32 = 13;
pub const REG_G_DX_ADDR: u32 = 14;
pub const REG_G_DY_ADDR: u32 = 15;
pub const REG_B_MIN_ADDR: u32 = 16;
pub const REG_B_DX_ADDR: u32 = 17;
pub const REG_B_DY_ADDR: u32 = 18;
pub const REG_A_MIN_ADDR: u32 = 19;
pub const REG_A_DX_ADDR: u32 = 20;
pub const REG_A_DY_ADDR: u32 = 21;
pub const REG_W_INVERSE_MIN_ADDR: u32 = 22;
pub const REG_W_INVERSE_DX_ADDR: u32 = 23;
pub const REG_W_INVERSE_DY_ADDR: u32 = 24;
pub const REG_S_MIN_ADDR: u32 = 25;
pub const REG_S_DX_ADDR: u32 = 26;
pub const REG_S_DY_ADDR: u32 = 27;
pub const REG_T_MIN_ADDR: u32 = 28;
pub const REG_T_DX_ADDR: u32 = 29;
pub const REG_T_DY_ADDR: u32 = 30;

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("ColorThrust");

    m.output("reg_bus_ready", m.high());
    let reg_bus_enable = m.input("reg_bus_enable", 1);
    let reg_bus_addr = m.input("reg_bus_addr", REG_BUS_ADDR_BIT_WIDTH);
    let reg_bus_write = m.input("reg_bus_write", 1);
    let reg_bus_write_data = m.input("reg_bus_write_data", 32);

    let reg_bus_write_enable = reg_bus_enable & reg_bus_write;

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
        let dy = m.reg(format!("{}_dy", name), num_bits);
        dy.drive_next(if_(reg_bus_write_enable & reg_bus_addr.eq(m.lit(dy_addr, REG_BUS_ADDR_BIT_WIDTH)), {
            reg_bus_write_data.bits(num_bits - 1, 0)
        }).else_({
            dy.value
        }));

        let row = m.reg(format!("{}_row", name), num_bits);

        let value = m.reg(name, num_bits);

        let (next_row, next_value) = if_(start, {
            (min.value, min.value)
        }).else_({
            if_(tile_x_last, {
                let next = row.value + dy.value;
                (next, next)
            }).else_({
                (row.value, value.value + dx.value)
            })
        });

        row.drive_next(next_row);

        value.drive_next(next_value);

        value.value
    };

    let w0 = interpolant("w0", 32, REG_W0_MIN_ADDR, REG_W0_DX_ADDR, REG_W0_DY_ADDR).bit(31);
    let w1 = interpolant("w1", 32, REG_W1_MIN_ADDR, REG_W1_DX_ADDR, REG_W1_DY_ADDR).bit(31);
    let w2 = interpolant("w2", 32, REG_W2_MIN_ADDR, REG_W2_DX_ADDR, REG_W2_DY_ADDR).bit(31);

    let r = interpolant("r", 24, REG_R_MIN_ADDR, REG_R_DX_ADDR, REG_R_DY_ADDR).bits(19, 12);
    let g = interpolant("g", 24, REG_G_MIN_ADDR, REG_G_DX_ADDR, REG_G_DY_ADDR).bits(19, 12);
    let b = interpolant("b", 24, REG_B_MIN_ADDR, REG_B_DX_ADDR, REG_B_DY_ADDR).bits(19, 12);
    let a = interpolant("a", 24, REG_A_MIN_ADDR, REG_A_DX_ADDR, REG_A_DY_ADDR).bits(19, 12);

    let w_inverse = interpolant("w_inverse", 32, REG_W_INVERSE_MIN_ADDR, REG_W_INVERSE_DX_ADDR, REG_W_INVERSE_DY_ADDR);

    let s = interpolant("s", 32, REG_S_MIN_ADDR, REG_S_DX_ADDR, REG_S_DY_ADDR).bits(31, 8);
    let t = interpolant("t", 32, REG_T_MIN_ADDR, REG_T_DX_ADDR, REG_T_DY_ADDR).bits(31, 8);

    generate_pixel_pipe(c);
    let pixel_pipe = m.instance("pixel_pipe", "PixelPipe");

    pixel_pipe.drive_input("start", start);

    pixel_pipe.drive_input("in_valid", input_generator_active.value);
    pixel_pipe.drive_input("in_last", tile_x_last & tile_y_last);
    pixel_pipe.drive_input("in_tile_addr", tile_y.value.concat(tile_x.value));

    pixel_pipe.drive_input("in_w0", w0);
    pixel_pipe.drive_input("in_w1", w1);
    pixel_pipe.drive_input("in_w2", w2);

    pixel_pipe.drive_input("in_r", r);
    pixel_pipe.drive_input("in_g", g);
    pixel_pipe.drive_input("in_b", b);
    pixel_pipe.drive_input("in_a", a);

    pixel_pipe.drive_input("in_w_inverse", w_inverse);

    pixel_pipe.drive_input("in_s", s);
    pixel_pipe.drive_input("in_t", t);

    m.output("reg_bus_read_data", m.lit(0u32, 31).concat(input_generator_active.value | pixel_pipe.output("active")));
    m.output("reg_bus_read_data_valid", reg_next_with_default("reg_bus_read_data_valid", reg_bus_enable & !reg_bus_write, false, m));

    m.output("color_buffer_bus_ready", m.high());
    let color_buffer_bus_enable = m.input("color_buffer_bus_enable", 1);
    let color_buffer_bus_addr = m.input("color_buffer_bus_addr", TILE_PIXELS_BITS);
    let color_buffer_bus_write = m.input("color_buffer_bus_write", 1);
    let color_buffer_bus_write_data = m.input("color_buffer_bus_write_data", 32);

    let color_buffer = m.mem("color_buffer", TILE_PIXELS_BITS, 32);
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
        color_buffer_bus_write_enable | pixel_pipe.output("color_buffer_write_port_enable"));

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
    m.output("color_buffer_bus_read_data_valid", reg_next_with_default("color_buffer_bus_read_data_valid", color_buffer_bus_read_enable, false, m));

    m.output("tex_buffer_bus_ready", m.high());
    let tex_buffer_bus_enable = m.input("tex_buffer_bus_enable", 1);
    let tex_buffer_bus_addr = m.input("tex_buffer_bus_addr", TILE_PIXELS_BITS);
    let tex_buffer_bus_write = m.input("tex_buffer_bus_write", 1);
    let tex_buffer_bus_write_data = m.input("tex_buffer_bus_write_data", 32);

    let tex_buffer = m.mem("tex_buffer", TILE_PIXELS_BITS, 32);
    let tex_buffer_bus_write_enable = tex_buffer_bus_enable & tex_buffer_bus_write;
    tex_buffer.write_port(tex_buffer_bus_addr, tex_buffer_bus_write_data, tex_buffer_bus_write_enable);

    let tex_buffer_bus_read_enable = tex_buffer_bus_enable & !tex_buffer_bus_write;
    let tex_buffer_read_port_value = tex_buffer.read_port(
        if_(tex_buffer_bus_read_enable, {
            tex_buffer_bus_addr
        }).else_({
            pixel_pipe.output("tex_buffer_read_port_addr")
        }),
        tex_buffer_bus_read_enable | pixel_pipe.output("tex_buffer_read_port_enable"));

    pixel_pipe.drive_input("tex_buffer_read_port_value", tex_buffer_read_port_value);

    m.output("tex_buffer_bus_read_data", tex_buffer_read_port_value);
    m.output("tex_buffer_bus_read_data_valid", reg_next_with_default("tex_buffer_bus_read_data_valid", tex_buffer_bus_read_enable, false, m));

    m
}

pub fn generate_pixel_pipe<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("PixelPipe");

    // Control
    let start = m.input("start", 1);

    let active = m.reg("active", 1);
    active.default_value(false);
    m.output("active", active.value);

    // Inputs
    let mut valid = m.input("in_valid", 1);
    let mut last = m.input("in_last", 1);
    let mut tile_addr = m.input("in_tile_addr", TILE_PIXELS_BITS);

    let w0 = m.input("in_w0", 1);
    let w1 = m.input("in_w1", 1);
    let w2 = m.input("in_w2", 1);
    let mut edge_test = !(w0 | w1 | w2);

    let mut r = m.input("in_r", 8);
    let mut g = m.input("in_g", 8);
    let mut b = m.input("in_b", 8);
    let mut a = m.input("in_a", 8);

    let w_inverse = m.input("in_w_inverse", 32);

    let mut s = m.input("in_s", 24);
    let mut t = m.input("in_t", 24);

    approx_reciprocal::generate(c, "WInverseReciprocal", W_INVERSE_FRACT_BITS - RESTORED_W_FRACT_BITS - 3, 4);
    let w_approx_reciprocal = m.instance("w_approx_reciprocal", "WInverseReciprocal");
    w_approx_reciprocal.drive_input("x", w_inverse);

    // Stages 1-9 (mostly just delay for w to arrive)
    for stage in 1..=9 {
        valid = reg_next_with_default(format!("stage_{}_valid", stage), valid, false, m);
        last = reg_next_with_default(format!("stage_{}_last", stage), last, false, m);
        tile_addr = reg_next(format!("stage_{}_tile_addr", stage), tile_addr, m);

        edge_test = reg_next(format!("stage_{}_edge_test", stage), edge_test, m);

        r = reg_next(format!("stage_{}_r", stage), r, m);
        g = reg_next(format!("stage_{}_g", stage), g, m);
        b = reg_next(format!("stage_{}_b", stage), b, m);
        a = reg_next(format!("stage_{}_a", stage), a, m);

        s = reg_next(format!("stage_{}_s", stage), s, m);
        t = reg_next(format!("stage_{}_t", stage), t, m);
    }

    //  Returned from issue before stage 1
    let w = w_approx_reciprocal.output("quotient");

    // Stage 10
    let valid = reg_next_with_default("stage_10_valid", valid, false, m);
    let last = reg_next_with_default("stage_10_last", last, false, m);
    let tile_addr = reg_next("stage_10_tile_addr", tile_addr, m);

    let edge_test = reg_next("stage_10_edge_test", edge_test, m);

    let r = reg_next("stage_10_r", r, m);
    let g = reg_next("stage_10_g", g, m);
    let b = reg_next("stage_10_b", b, m);
    let a = reg_next("stage_10_a", a, m);

    let s = reg_next("stage_10_s", s, m);
    let t = reg_next("stage_10_t", t, m);

    let w = reg_next("stage_10_w", w, m);

    let s = s * w;
    let t = t * w;

    // Stage 11
    let valid = reg_next_with_default("stage_11_valid", valid, false, m);
    let last = reg_next_with_default("stage_11_last", last, false, m);
    let tile_addr = reg_next("stage_11_tile_addr", tile_addr, m);

    let edge_test = reg_next("stage_11_edge_test", edge_test, m);

    let r = reg_next("stage_11_r", r, m);
    let g = reg_next("stage_11_g", g, m);
    let b = reg_next("stage_11_b", b, m);
    let a = reg_next("stage_11_a", a, m);

    let s = reg_next("stage_11_s", s, m);
    let t = reg_next("stage_11_t", t, m);

    let s_floor = s.bits(31, ST_FRACT_BITS);
    let t_floor = t.bits(31, ST_FRACT_BITS);
    let s_fract = s.bits(ST_FRACT_BITS - 1, ST_FRACT_BITS - ST_FILTER_BITS);
    let t_fract = t.bits(ST_FRACT_BITS - 1, ST_FRACT_BITS - ST_FILTER_BITS);
    let one_minus_s_fract = m.high().concat(m.lit(0u32, ST_FILTER_BITS)) - m.low().concat(s_fract);
    let one_minus_t_fract = m.high().concat(m.lit(0u32, ST_FILTER_BITS)) - m.low().concat(t_fract);

    //  Issue color buffer read for prev_color
    m.output("color_buffer_read_port_addr", tile_addr);
    m.output("color_buffer_read_port_enable", valid);

    //  Issue tex buffer read for unfiltered texel
    m.output("tex_buffer_read_port_addr", t_floor.bits(3, 0).concat(s_floor.bits(3, 0)));
    m.output("tex_buffer_read_port_enable", valid);

    // Stage 12
    let valid = reg_next_with_default("stage_12_valid", valid, false, m);
    let last = reg_next_with_default("stage_12_last", last, false, m);
    let tile_addr = reg_next("stage_12_tile_addr", tile_addr, m);

    let edge_test = reg_next("stage_12_edge_test", edge_test, m);

    let r = reg_next("stage_12_r", r, m);
    let g = reg_next("stage_12_g", g, m);
    let b = reg_next("stage_12_b", b, m);
    let a = reg_next("stage_12_a", a, m);

    let s_floor = reg_next("stage_12_s_floor", s_floor, m);
    let t_floor = reg_next("stage_12_t_floor", t_floor, m);
    let s_fract = reg_next("stage_12_s_fract", s_fract, m);
    let t_fract = reg_next("stage_12_t_fract", t_fract, m);
    let one_minus_s_fract = reg_next("stage_12_one_minus_s_fract", one_minus_s_fract, m);
    let one_minus_t_fract = reg_next("stage_12_one_minus_t_fract", one_minus_t_fract, m);

    //  Returned from issue in previous stage
    let prev_color = m.input("color_buffer_read_port_value", 32);

    //  Returned from issue in previous stage
    let texel = m.input("tex_buffer_read_port_value", 32);

    let texel_r = texel.bits(23, 16);
    let texel_g = texel.bits(15, 8);
    let texel_b = texel.bits(7, 0);
    let texel_a = texel.bits(31, 24);
    let r = (r * texel_r).bits(15, 8);
    let g = (g * texel_g).bits(15, 8);
    let b = (b * texel_b).bits(15, 8);
    let a = (a * texel_a).bits(15, 8);
    /*let r = s_floor;
    let g = t_floor;*/
    let color = a.concat(r).concat(g).concat(b);

    // Stage 13
    let valid = reg_next_with_default("stage_13_valid", valid, false, m);
    let last = reg_next_with_default("stage_13_last", last, false, m);
    let tile_addr = reg_next("stage_13_tile_addr", tile_addr, m);

    let edge_test = reg_next("stage_13_edge_test", edge_test, m);

    let color = reg_next("stage_13_color", color, m);

    m.output("color_buffer_write_port_addr", tile_addr);
    m.output("color_buffer_write_port_value", color);
    m.output("color_buffer_write_port_enable", valid & edge_test);

    active.drive_next(if_(start, {
        m.high()
    }).else_if(valid & last, {
        m.low()
    }).else_({
        active.value
    }));

    m
}
