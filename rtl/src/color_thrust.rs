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

    let busy = m.reg("busy", 1);
    busy.default_value(false);

    m.output("reg_bus_read_data", m.lit(0u32, 31).concat(busy.value));
    m.output("reg_bus_read_data_valid", reg_next_with_default("reg_bus_read_data_valid", reg_bus_enable & !reg_bus_write, false, m));

    let tile_x = m.reg("tile_x", TILE_DIM_BITS);
    let tile_y = m.reg("tile_y", TILE_DIM_BITS);

    let (next_busy, next_tile_x, next_tile_y) = if_(reg_bus_write_enable & reg_bus_addr.eq(m.lit(REG_START_ADDR, reg_bus_addr_bit_width)), {
        let next_busy = m.high();

        let next_tile_x = m.lit(0u32, TILE_DIM_BITS);
        let next_tile_y = m.lit(0u32, TILE_DIM_BITS);

        (next_busy, next_tile_x, next_tile_y)
    }).else_({
        let next_busy = busy.value;

        let next_tile_x = tile_x.value + m.lit(1u32, TILE_DIM_BITS);
        let next_tile_y = tile_y.value;

        let (next_busy, next_tile_y) = if_(tile_x.value.eq(m.lit(TILE_DIM - 1, TILE_DIM_BITS)), {
            let next_busy = if_(tile_y.value.eq(m.lit(TILE_DIM - 1, TILE_DIM_BITS)), {
                m.low()
            }).else_({
                next_busy
            });

            let next_tile_y = tile_y.value + m.lit(1u32, TILE_DIM_BITS);

            (next_busy, next_tile_y)
        }).else_({
            (next_busy, next_tile_y)
        });

        (next_busy, next_tile_x, next_tile_y)
    });

    busy.drive_next(next_busy);

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

        let (next_row, next_value) = if_(reg_bus_write_enable & reg_bus_addr.eq(m.lit(REG_START_ADDR, reg_bus_addr_bit_width)), {
            (min.value, min.value)
        }).else_({
            if_(tile_x.value.eq(m.lit(TILE_DIM - 1, TILE_DIM_BITS)), {
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

    let w0 = interpolant("w0", 32, REG_W0_MIN_ADDR, REG_W0_DX_ADDR, REG_W0_DY_ADDR);
    let w1 = interpolant("w1", 32, REG_W1_MIN_ADDR, REG_W1_DX_ADDR, REG_W1_DY_ADDR);
    let w2 = interpolant("w2", 32, REG_W2_MIN_ADDR, REG_W2_DX_ADDR, REG_W2_DY_ADDR);

    let r = interpolant("r", 24, REG_R_MIN_ADDR, REG_R_DX_ADDR, REG_R_DY_ADDR).bits(19, 12);
    let g = interpolant("g", 24, REG_G_MIN_ADDR, REG_G_DX_ADDR, REG_G_DY_ADDR).bits(19, 12);
    let b = interpolant("b", 24, REG_B_MIN_ADDR, REG_B_DX_ADDR, REG_B_DY_ADDR).bits(19, 12);
    let a = interpolant("a", 24, REG_A_MIN_ADDR, REG_A_DX_ADDR, REG_A_DY_ADDR).bits(19, 12);

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
            tile_y.value.concat(tile_x.value)
        }),
        if_(color_buffer_bus_write_enable, {
            color_buffer_bus_write_data
        }).else_({
            a.concat(r).concat(g).concat(b)
        }),
        color_buffer_bus_write_enable | (busy.value & !(w0.bit(31) | w1.bit(31) | w2.bit(31))));

    let color_buffer_bus_read_enable = color_buffer_bus_enable & !color_buffer_bus_write;
    m.output("color_buffer_bus_read_data", color_buffer.read_port(color_buffer_bus_addr, color_buffer_bus_read_enable));
    m.output("color_buffer_bus_read_data_valid", reg_next_with_default("color_buffer_bus_read_data_valid", color_buffer_bus_read_enable, false, m));

    m
}
