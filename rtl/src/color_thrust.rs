use crate::helpers::*;

use kaze::*;

pub const TILE_DIM_BITS: u32 = 4;
pub const TILE_DIM: u32 = 1 << TILE_DIM_BITS;
pub const TILE_PIXELS_BITS: u32 = TILE_DIM_BITS * 2;
pub const TILE_PIXELS: u32 = 1 << TILE_PIXELS_BITS;

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

pub fn generate<'a>(c: &'a Context<'a>) -> &Module<'a> {
    let m = c.module("ColorThrust");

    m.output("reg_bus_ready", m.high());
    let reg_bus_enable = m.input("reg_bus_enable", 1);
    let reg_bus_addr_bit_width = 5;
    let reg_bus_addr = m.input("reg_bus_addr", reg_bus_addr_bit_width);
    let reg_bus_write = m.input("reg_bus_write", 1);
    let reg_bus_write_data = m.input("reg_bus_write_data", 32);

    let reg_bus_write_enable = reg_bus_enable & reg_bus_write;

    let input_generator_active = m.reg("input_generator_active", 1);
    input_generator_active.default_value(false);

    let tile_x = m.reg("tile_x", TILE_DIM_BITS);
    let tile_y = m.reg("tile_y", TILE_DIM_BITS);
    let tile_x_last = tile_x.value.eq(m.lit(TILE_DIM - 1, TILE_DIM_BITS));
    let tile_y_last = tile_y.value.eq(m.lit(TILE_DIM - 1, TILE_DIM_BITS));

    let start = reg_bus_write_enable & reg_bus_addr.eq(m.lit(REG_START_ADDR, reg_bus_addr_bit_width));

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
        min.drive_next(if_(reg_bus_write_enable & reg_bus_addr.eq(m.lit(min_addr, reg_bus_addr_bit_width)), {
            reg_bus_write_data.bits(num_bits - 1, 0)
        }).else_({
            min.value
        }));
        let dx = m.reg(format!("{}_dx", name), num_bits);
        dx.drive_next(if_(reg_bus_write_enable & reg_bus_addr.eq(m.lit(dx_addr, reg_bus_addr_bit_width)), {
            reg_bus_write_data.bits(num_bits - 1, 0)
        }).else_({
            dx.value
        }));
        let dy = m.reg(format!("{}_dy", name), num_bits);
        dy.drive_next(if_(reg_bus_write_enable & reg_bus_addr.eq(m.lit(dy_addr, reg_bus_addr_bit_width)), {
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
    let valid = m.input("in_valid", 1);
    let last = m.input("in_last", 1);
    let tile_addr = m.input("in_tile_addr", TILE_PIXELS_BITS);

    let w0 = m.input("in_w0", 1);
    let w1 = m.input("in_w1", 1);
    let w2 = m.input("in_w2", 1);
    let edge_test = !(w0 | w1 | w2);

    let r = m.input("in_r", 8);
    let g = m.input("in_g", 8);
    let b = m.input("in_b", 8);
    let a = m.input("in_a", 8);

    // Stage 1
    let valid = reg_next_with_default("stage_1_valid", valid, false, m);
    let last = reg_next_with_default("stage_1_last", last, false, m);
    let tile_addr = reg_next("stage_1_tile_addr", tile_addr, m);

    let edge_test = reg_next("stage_1_edge_test", edge_test, m);

    let r = reg_next("stage_1_r", r, m);
    let g = reg_next("stage_1_g", g, m);
    let b = reg_next("stage_1_b", b, m);
    let a = reg_next("stage_1_a", a, m);

    //  Issue color buffer read for prev_color
    m.output("color_buffer_read_port_addr", tile_addr);
    m.output("color_buffer_read_port_enable", valid);

    // Stage 2
    let valid = reg_next_with_default("stage_2_valid", valid, false, m);
    let last = reg_next_with_default("stage_2_last", last, false, m);
    let tile_addr = reg_next("stage_2_tile_addr", tile_addr, m);

    let edge_test = reg_next("stage_2_edge_test", edge_test, m);

    let r = reg_next("stage_2_r", r, m);
    let g = reg_next("stage_2_g", g, m);
    let b = reg_next("stage_2_b", b, m);
    let a = reg_next("stage_2_a", a, m);

    //  Returned from issue in previous stage
    let prev_color = m.input("color_buffer_read_port_value", 32);

    // Blending test :)
    let r = (m.low().concat(r) + m.low().concat(prev_color.bits(23, 16))).bits(8, 1);
    let g = (m.low().concat(g) + m.low().concat(prev_color.bits(15, 8))).bits(8, 1);
    let b = (m.low().concat(b) + m.low().concat(prev_color.bits(7, 0))).bits(8, 1);
    let a = (m.low().concat(a) + m.low().concat(prev_color.bits(31, 24))).bits(8, 1);
    let color = a.concat(r).concat(g).concat(b);

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
