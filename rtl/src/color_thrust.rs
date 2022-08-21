mod tex_cache;

use tex_cache::*;

use crate::approx_reciprocal::*;
use crate::buster::*;
use crate::flow_controlled_pipe::*;
use crate::word_mem::*;

use color_thrust_interface::params_and_regs::*;

use kaze::*;

pub struct ColorThrust<'a> {
    pub m: &'a Module<'a>,

    pub reg_port: ReplicaPort<'a>,
    pub color_buffer_port: ReplicaPort<'a>,
    pub depth_buffer_port: ReplicaPort<'a>,
    pub tex_cache_system_port: PrimaryPort<'a>,
}

impl<'a> ColorThrust<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> ColorThrust<'a> {
        let m = p.module(instance_name, "ColorThrust");

        let reg_bus_ready = m.output("reg_bus_ready", m.high());
        let reg_bus_enable = m.input("reg_bus_enable", 1);
        let reg_bus_addr = m.input("reg_bus_addr", REG_BUS_ADDR_BITS);
        let truncated_reg_bus_addr = reg_bus_addr.bits(REG_BUS_ADDR_BIT_WIDTH - 1, 0);
        let reg_bus_write = m.input("reg_bus_write", 1);
        let reg_bus_write_data = m.input("reg_bus_write_data", 128);

        let reg_bus_write_enable = reg_bus_enable & reg_bus_write;

        let tex_cache_invalidate = reg_bus_write_enable & truncated_reg_bus_addr.eq(m.lit(REG_TEX_CACHE_INVALIDATE_ADDR, REG_BUS_ADDR_BIT_WIDTH));

        let reg_depth_settings = m.reg("depth_settings", REG_DEPTH_SETTINGS_BITS);
        reg_depth_settings.drive_next(if_(reg_bus_write_enable & truncated_reg_bus_addr.eq(m.lit(REG_DEPTH_SETTINGS_ADDR, REG_BUS_ADDR_BIT_WIDTH)), {
            reg_bus_write_data.bits(REG_DEPTH_SETTINGS_BITS - 1, 0)
        }).else_({
            reg_depth_settings
        }));
        let depth_test_enable = reg_depth_settings.bit(REG_DEPTH_TEST_ENABLE_BIT);
        let depth_write_mask_enable = reg_depth_settings.bit(REG_DEPTH_WRITE_MASK_ENABLE_BIT);

        let reg_texture_settings = m.reg("texture_settings", REG_TEXTURE_SETTINGS_BITS);
        reg_texture_settings.drive_next(if_(reg_bus_write_enable & truncated_reg_bus_addr.eq(m.lit(REG_TEXTURE_SETTINGS_ADDR, REG_BUS_ADDR_BIT_WIDTH)), {
            reg_bus_write_data.bits(REG_TEXTURE_SETTINGS_BITS - 1, 0)
        }).else_({
            reg_texture_settings
        }));
        let tex_filter_select = reg_texture_settings.bit(REG_TEXTURE_SETTINGS_FILTER_SELECT_BIT_OFFSET);
        let tex_dim = reg_texture_settings.bits(REG_TEXTURE_SETTINGS_DIM_BIT_OFFSET + REG_TEXTURE_SETTINGS_DIM_BITS - 1, REG_TEXTURE_SETTINGS_DIM_BIT_OFFSET);

        let reg_texture_base = m.reg("texture_base", REG_TEXTURE_BASE_BITS);
        reg_texture_base.drive_next(if_(reg_bus_write_enable & truncated_reg_bus_addr.eq(m.lit(REG_TEXTURE_BASE_ADDR, REG_BUS_ADDR_BIT_WIDTH)), {
            reg_bus_write_data.bits(6 + 4 + REG_TEXTURE_BASE_BITS - 1, 6 + 4)
        }).else_({
            reg_texture_base
        }));

        let reg_blend_settings = m.reg("blend_settings", REG_BLEND_SETTINGS_BITS);
        reg_blend_settings.drive_next(if_(reg_bus_write_enable & truncated_reg_bus_addr.eq(m.lit(REG_BLEND_SETTINGS_ADDR, REG_BUS_ADDR_BIT_WIDTH)), {
            reg_bus_write_data.bits(REG_BLEND_SETTINGS_BITS - 1, 0)
        }).else_({
            reg_blend_settings
        }));
        let blend_src_factor = reg_blend_settings.bits(REG_BLEND_SETTINGS_SRC_FACTOR_BIT_OFFSET + REG_BLEND_SETTINGS_SRC_FACTOR_BITS - 1, REG_BLEND_SETTINGS_SRC_FACTOR_BIT_OFFSET);
        let blend_dst_factor = reg_blend_settings.bits(REG_BLEND_SETTINGS_DST_FACTOR_BIT_OFFSET + REG_BLEND_SETTINGS_DST_FACTOR_BITS - 1, REG_BLEND_SETTINGS_DST_FACTOR_BIT_OFFSET);

        let input_generator_active = m.reg("input_generator_active", 1);
        input_generator_active.default_value(false);

        let tile_x = m.reg("tile_x", TILE_DIM_BITS);
        let tile_y = m.reg("tile_y", TILE_DIM_BITS);
        let tile_x_last = tile_x.eq(m.lit(TILE_DIM - 1, TILE_DIM_BITS));
        let tile_y_last = tile_y.eq(m.lit(TILE_DIM - 1, TILE_DIM_BITS));

        let start = reg_bus_write_enable & truncated_reg_bus_addr.eq(m.lit(REG_START_ADDR, REG_BUS_ADDR_BIT_WIDTH));

        let pixel_pipe = PixelPipe::new("pixel_pipe", m);

        pixel_pipe.start.drive(start);

        pixel_pipe.depth_test_enable.drive(depth_test_enable);
        pixel_pipe.depth_write_mask_enable.drive(depth_write_mask_enable);

        pixel_pipe.tex_filter_select.drive(tex_filter_select);
        pixel_pipe.tex_dim.drive(tex_dim);
        pixel_pipe.tex_base.drive(reg_texture_base);

        pixel_pipe.blend_src_factor.drive(blend_src_factor);
        pixel_pipe.blend_dst_factor.drive(blend_dst_factor);

        pixel_pipe.in_valid.drive(input_generator_active);
        pixel_pipe.in_tile_addr.drive(tile_y.concat(tile_x));

        let (next_input_generator_active, next_tile_x, next_tile_y) = if_(start, {
            let next_input_generator_active = m.high();

            let next_tile_x = m.lit(0u32, TILE_DIM_BITS);
            let next_tile_y = m.lit(0u32, TILE_DIM_BITS);

            (next_input_generator_active, next_tile_x, next_tile_y)
        }).else_if(pixel_pipe.in_ready, {
            let next_input_generator_active = input_generator_active;

            let next_tile_x = tile_x + m.lit(1u32, TILE_DIM_BITS);
            let next_tile_y = tile_y;

            let (next_input_generator_active, next_tile_y) = if_(tile_x_last, {
                let next_input_generator_active = if_(tile_y_last, {
                    m.low()
                }).else_({
                    next_input_generator_active
                });

                let next_tile_y = tile_y + m.lit(1u32, TILE_DIM_BITS);

                (next_input_generator_active, next_tile_y)
            }).else_({
                (next_input_generator_active, next_tile_y)
            });

            (next_input_generator_active, next_tile_x, next_tile_y)
        }).else_({
            (input_generator_active, tile_x, tile_y)
        });

        input_generator_active.drive_next(next_input_generator_active);

        tile_x.drive_next(next_tile_x);
        tile_y.drive_next(next_tile_y);

        let interpolant = |name: &str, num_bits, min_addr: u32, dx_addr: u32, dy_addr: u32| {
            let min = m.reg(format!("{}_min", name), num_bits);
            min.drive_next(if_(reg_bus_write_enable & truncated_reg_bus_addr.eq(m.lit(min_addr, REG_BUS_ADDR_BIT_WIDTH)), {
                reg_bus_write_data.bits(num_bits - 1, 0)
            }).else_({
                min
            }));
            let dx = m.reg(format!("{}_dx", name), num_bits);
            dx.drive_next(if_(reg_bus_write_enable & truncated_reg_bus_addr.eq(m.lit(dx_addr, REG_BUS_ADDR_BIT_WIDTH)), {
                reg_bus_write_data.bits(num_bits - 1, 0)
            }).else_({
                dx
            }));
            let dx_mirror = m.reg(format!("{}_dx_mirror", name), num_bits);
            dx_mirror.drive_next(if_(start, {
                dx
            }).else_({
                dx_mirror
            }));
            let dy = m.reg(format!("{}_dy", name), num_bits);
            dy.drive_next(if_(reg_bus_write_enable & truncated_reg_bus_addr.eq(m.lit(dy_addr, REG_BUS_ADDR_BIT_WIDTH)), {
                reg_bus_write_data.bits(num_bits - 1, 0)
            }).else_({
                dy
            }));
            let dy_mirror = m.reg(format!("{}_dy_mirror", name), num_bits);
            dy_mirror.drive_next(if_(start, {
                dy
            }).else_({
                dy_mirror
            }));

            let row = m.reg(format!("{}_row", name), num_bits);

            let value = m.reg(name, num_bits);

            let (next_row, next_value) = if_(start, {
                (min.into(), min.into())
            }).else_if(pixel_pipe.in_ready, {
                if_(tile_x_last, {
                    let next = row + dy_mirror;
                    (next, next)
                }).else_({
                    (row, value + dx_mirror)
                })
            }).else_({
                (row, value)
            });

            row.drive_next(next_row);

            value.drive_next(next_value);

            value
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

        pixel_pipe.in_w0.drive(w0);
        pixel_pipe.in_w1.drive(w1);
        pixel_pipe.in_w2.drive(w2);

        pixel_pipe.in_r.drive(r);
        pixel_pipe.in_g.drive(g);
        pixel_pipe.in_b.drive(b);
        pixel_pipe.in_a.drive(a);

        pixel_pipe.in_w_inverse.drive(w_inverse);

        pixel_pipe.in_z.drive(z);

        pixel_pipe.in_s.drive(s);
        pixel_pipe.in_t.drive(t);

        let reg_bus_read_data = m.output("reg_bus_read_data", m.lit(0u32, 127).concat(input_generator_active | pixel_pipe.active));
        let reg_bus_read_data_valid = m.output("reg_bus_read_data_valid", (reg_bus_enable & !reg_bus_write).reg_next_with_default("reg_bus_read_data_valid", false));

        let color_buffer_bus_ready = m.output("color_buffer_bus_ready", m.high());
        let color_buffer_bus_enable = m.input("color_buffer_bus_enable", 1);
        let color_buffer_bus_addr = m.input("color_buffer_bus_addr", 20);
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
                color_buffer_bus_addr.bits(TILE_PIXELS_WORDS_BITS - 1, 0)
            }).else_({
                pixel_pipe.color_buffer_write_port_addr
            }),
            if_(color_buffer_bus_write_enable, {
                color_buffer_bus_write_data
            }).else_({
                pixel_pipe.color_buffer_write_port_value
            }),
            color_buffer_bus_write_enable | pixel_pipe.color_buffer_write_port_enable,
            if_(color_buffer_bus_write_enable, {
                color_buffer_bus_write_word_enable
            }).else_({
                pixel_pipe.color_buffer_write_port_word_enable
            }));

        let color_buffer_bus_read_enable = color_buffer_bus_enable & !color_buffer_bus_write;
        let color_buffer_read_port_value = color_buffer.read_port(
            if_(color_buffer_bus_read_enable, {
                color_buffer_bus_addr.bits(TILE_PIXELS_WORDS_BITS - 1, 0)
            }).else_({
                pixel_pipe.color_buffer_read_port_addr
            }),
            color_buffer_bus_read_enable | pixel_pipe.color_buffer_read_port_enable);

        pixel_pipe.color_buffer_read_port_value.drive(color_buffer_read_port_value);

        let color_buffer_bus_read_data = m.output("color_buffer_bus_read_data", color_buffer_read_port_value);
        let color_buffer_bus_read_data_valid = m.output("color_buffer_bus_read_data_valid", color_buffer_bus_read_enable.reg_next_with_default("color_buffer_bus_read_data_valid", false));

        let depth_buffer_bus_ready = m.output("depth_buffer_bus_ready", m.high());
        let depth_buffer_bus_enable = m.input("depth_buffer_bus_enable", 1);
        let depth_buffer_bus_addr = m.input("depth_buffer_bus_addr", 20);
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
                depth_buffer_bus_addr.bits(TILE_PIXELS_WORDS_BITS - 1 - 1, 0)
            }).else_({
                pixel_pipe.depth_buffer_write_port_addr
            }),
            if_(depth_buffer_bus_write_enable, {
                depth_buffer_bus_write_data
            }).else_({
                pixel_pipe.depth_buffer_write_port_value
            }),
            depth_buffer_bus_write_enable | pixel_pipe.depth_buffer_write_port_enable,
            if_(depth_buffer_bus_write_enable, {
                depth_buffer_bus_write_word_enable
            }).else_({
                pixel_pipe.depth_buffer_write_port_word_enable
            }));

        let depth_buffer_bus_read_enable = depth_buffer_bus_enable & !depth_buffer_bus_write;
        let depth_buffer_read_port_value = depth_buffer.read_port(
            if_(depth_buffer_bus_read_enable, {
                depth_buffer_bus_addr.bits(TILE_PIXELS_WORDS_BITS - 1 - 1, 0)
            }).else_({
                pixel_pipe.depth_buffer_read_port_addr
            }),
            depth_buffer_bus_read_enable | pixel_pipe.depth_buffer_read_port_enable);

        pixel_pipe.depth_buffer_read_port_value.drive(depth_buffer_read_port_value);

        let depth_buffer_bus_read_data = m.output("depth_buffer_bus_read_data", depth_buffer_read_port_value);
        let depth_buffer_bus_read_data_valid = m.output("depth_buffer_bus_read_data_valid", depth_buffer_bus_read_enable.reg_next_with_default("depth_buffer_bus_read_data_valid", false));

        pixel_pipe.tex_cache_invalidate.drive(tex_cache_invalidate);
        let tex_cache_system_port = pixel_pipe.tex_cache_system_port.forward("tex_cache_system", m);

        ColorThrust {
            m,

            reg_port: ReplicaPort {
                bus_enable: reg_bus_enable,
                bus_addr: reg_bus_addr,
                bus_write: reg_bus_write,
                bus_write_data: reg_bus_write_data,
                bus_write_byte_enable: m.input("reg_bus_write_byte_enable", 128 / 8),
                bus_ready: reg_bus_ready,
                bus_read_data: reg_bus_read_data,
                bus_read_data_valid: reg_bus_read_data_valid,
            },
            color_buffer_port: ReplicaPort {
                bus_enable: color_buffer_bus_enable,
                bus_addr: color_buffer_bus_addr,
                bus_write: color_buffer_bus_write,
                bus_write_data: color_buffer_bus_write_data,
                bus_write_byte_enable: color_buffer_bus_write_byte_enable,
                bus_ready: color_buffer_bus_ready,
                bus_read_data: color_buffer_bus_read_data,
                bus_read_data_valid: color_buffer_bus_read_data_valid,
            },
            depth_buffer_port: ReplicaPort {
                bus_enable: depth_buffer_bus_enable,
                bus_addr: depth_buffer_bus_addr,
                bus_write: depth_buffer_bus_write,
                bus_write_data: depth_buffer_bus_write_data,
                bus_write_byte_enable: depth_buffer_bus_write_byte_enable,
                bus_ready: depth_buffer_bus_ready,
                bus_read_data: depth_buffer_bus_read_data,
                bus_read_data_valid: depth_buffer_bus_read_data_valid,
            },
            tex_cache_system_port,
        }
    }
}

pub struct PixelPipe<'a> {
    pub m: &'a Module<'a>,

    // Control
    pub start: &'a Input<'a>,
    pub active: &'a Output<'a>,

    // Inputs
    pub in_valid: &'a Input<'a>,
    pub in_tile_addr: &'a Input<'a>,

    pub in_w0: &'a Input<'a>,
    pub in_w1: &'a Input<'a>,
    pub in_w2: &'a Input<'a>,

    pub in_r: &'a Input<'a>,
    pub in_g: &'a Input<'a>,
    pub in_b: &'a Input<'a>,
    pub in_a: &'a Input<'a>,

    pub in_w_inverse: &'a Input<'a>,

    pub in_z: &'a Input<'a>,

    pub in_s: &'a Input<'a>,
    pub in_t: &'a Input<'a>,

    pub tex_cache_system_port: PrimaryPort<'a>,

    // Aux inputs
    pub depth_test_enable: &'a Input<'a>,
    pub depth_buffer_read_port_value: &'a Input<'a>,

    pub tex_filter_select: &'a Input<'a>,
    pub tex_dim: &'a Input<'a>,
    pub tex_base: &'a Input<'a>,

    pub tex_cache_invalidate: &'a Input<'a>,

    pub depth_write_mask_enable: &'a Input<'a>,
    pub blend_src_factor: &'a Input<'a>,
    pub blend_dst_factor: &'a Input<'a>,
    pub color_buffer_read_port_value: &'a Input<'a>,

    // Outputs
    pub in_ready: &'a Output<'a>,

    // Aux outputs
    pub depth_buffer_read_port_addr: &'a Output<'a>,
    pub depth_buffer_read_port_enable: &'a Output<'a>,

    pub color_buffer_read_port_addr: &'a Output<'a>,
    pub color_buffer_read_port_enable: &'a Output<'a>,

    pub color_buffer_write_port_addr: &'a Output<'a>,
    pub color_buffer_write_port_value: &'a Output<'a>,
    pub color_buffer_write_port_enable: &'a Output<'a>,
    pub color_buffer_write_port_word_enable: &'a Output<'a>,

    pub depth_buffer_write_port_addr: &'a Output<'a>,
    pub depth_buffer_write_port_value: &'a Output<'a>,
    pub depth_buffer_write_port_enable: &'a Output<'a>,
    pub depth_buffer_write_port_word_enable: &'a Output<'a>,
}

impl<'a> PixelPipe<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> PixelPipe<'a> {
        let m = p.module(instance_name, "PixelPipe");

        // Control
        let start = m.input("start", 1);

        let active = m.reg("active", 1);
        active.default_value(false);

        let finished_pixel_acc = m.reg("finished_pixel_acc", TILE_PIXELS_BITS + 1);

        // Inputs
        let in_valid = m.input("in_valid", 1);
        let in_tile_addr = m.input("in_tile_addr", TILE_PIXELS_BITS);

        let in_w0 = m.input("in_w0", 1);
        let in_w1 = m.input("in_w1", 1);
        let in_w2 = m.input("in_w2", 1);

        let in_r = m.input("in_r", COLOR_WHOLE_BITS);
        let in_g = m.input("in_g", COLOR_WHOLE_BITS);
        let in_b = m.input("in_b", COLOR_WHOLE_BITS);
        let in_a = m.input("in_a", COLOR_WHOLE_BITS);

        let in_w_inverse = m.input("in_w_inverse", 32);

        let in_z = m.input("in_z", 16);

        let in_s = m.input("in_s", 32 - RESTORED_W_FRACT_BITS);
        let in_t = m.input("in_t", 32 - RESTORED_W_FRACT_BITS);

        let valid = in_valid;
        let tile_addr = in_tile_addr;

        //  Reject pixel if it doesn't pass edge test before it enters the first pipe
        let w0 = in_w0;
        let w1 = in_w1;
        let w2 = in_w2;
        let edge_test = !(w0 | w1 | w2);
        let edge_test_reject = valid & !edge_test;
        let valid = valid & edge_test;

        // Clamp negative color values early so the pipeline only has to deal with unsigned values
        let clamp_comp = |comp: &'a dyn Signal<'a>| -> &'a dyn Signal<'a> {
            if_(comp.bit(COLOR_WHOLE_BITS - 1), {
                m.lit(0u32, COLOR_WHOLE_BITS - 1)
            }).else_({
                comp.bits(COLOR_WHOLE_BITS - 2, 0)
            })
        };
        let r = clamp_comp(in_r);
        let g = clamp_comp(in_g);
        let b = clamp_comp(in_b);
        let a = clamp_comp(in_a);

        let w_inverse = in_w_inverse;

        let z = in_z;

        let s = in_s;
        let t = in_t;

        // Depth test pipe
        let depth_test_pipe = m.module("depth_test_pipe", "FlowControlledDepthTestPipe");
        let depth_test_pipe_inner = DepthTestPipe::new("depth_test_pipe_inner", depth_test_pipe);
        let mut depth_test_pipe = FlowControlledPipe::new(
            depth_test_pipe,
            2,
            depth_test_pipe_inner.in_valid,
            depth_test_pipe_inner.out_valid);

        //  Aux
        let depth_test_enable = m.input("depth_test_enable", 1);
        depth_test_pipe.aux_input("depth_test_enable", depth_test_pipe_inner.depth_test_enable).drive(depth_test_enable);

        let depth_buffer_read_port_addr = m.output("depth_buffer_read_port_addr", depth_test_pipe.aux_output("depth_buffer_read_port_addr", depth_test_pipe_inner.depth_buffer_read_port_addr));
        let depth_buffer_read_port_enable = m.output("depth_buffer_read_port_enable", depth_test_pipe.aux_output("depth_buffer_read_port_enable", depth_test_pipe_inner.depth_buffer_read_port_enable));

        let depth_buffer_read_port_value = m.input("depth_buffer_read_port_value", 128);
        depth_test_pipe.aux_input("depth_buffer_read_port_value", depth_test_pipe_inner.depth_buffer_read_port_value).drive(depth_buffer_read_port_value);

        //  Inputs
        depth_test_pipe.in_valid.drive(valid);

        depth_test_pipe.input("in_tile_addr", depth_test_pipe_inner.in_tile_addr).drive(tile_addr);

        depth_test_pipe.input("in_r", depth_test_pipe_inner.in_r).drive(r);
        depth_test_pipe.input("in_g", depth_test_pipe_inner.in_g).drive(g);
        depth_test_pipe.input("in_b", depth_test_pipe_inner.in_b).drive(b);
        depth_test_pipe.input("in_a", depth_test_pipe_inner.in_a).drive(a);

        depth_test_pipe.input("in_w_inverse", depth_test_pipe_inner.in_w_inverse).drive(w_inverse);

        depth_test_pipe.input("in_z", depth_test_pipe_inner.in_z).drive(z);

        depth_test_pipe.input("in_s", depth_test_pipe_inner.in_s).drive(s);
        depth_test_pipe.input("in_t", depth_test_pipe_inner.in_t).drive(t);

        //  Outputs
        let tile_addr = depth_test_pipe.output("out_tile_addr", depth_test_pipe_inner.out_tile_addr);

        let r = depth_test_pipe.output("out_r", depth_test_pipe_inner.out_r);
        let g = depth_test_pipe.output("out_g", depth_test_pipe_inner.out_g);
        let b = depth_test_pipe.output("out_b", depth_test_pipe_inner.out_b);
        let a = depth_test_pipe.output("out_a", depth_test_pipe_inner.out_a);

        let w_inverse = depth_test_pipe.output("out_w_inverse", depth_test_pipe_inner.out_w_inverse);

        let z = depth_test_pipe.output("out_z", depth_test_pipe_inner.out_z);

        let s = depth_test_pipe.output("out_s", depth_test_pipe_inner.out_s);
        let t = depth_test_pipe.output("out_t", depth_test_pipe_inner.out_t);

        let depth_test_result = depth_test_pipe.output("out_depth_test_result", depth_test_pipe_inner.out_depth_test_result);

        let in_ready = m.output("in_ready", depth_test_pipe.in_ready | edge_test_reject);

        // TODO: I don't like that these valid signals have moved
        let valid = depth_test_pipe.out_valid.unwrap();

        // Reject pixel if it doesn't pass depth test before entering the next pipe
        let depth_test_reject = valid & !depth_test_result;
        let valid = valid & depth_test_result;

        // Front pipe
        let front_pipe = m.module("front_pipe", "FlowControlledFrontPipe");
        let front_pipe_inner = FrontPipe::new("front_pipe_inner", front_pipe);
        let mut front_pipe = FlowControlledPipe::new(
            front_pipe,
            15,
            front_pipe_inner.in_valid,
            front_pipe_inner.out_valid);

        //  Aux
        let tex_filter_select = m.input("tex_filter_select", 1);
        front_pipe.aux_input("tex_filter_select", front_pipe_inner.tex_filter_select).drive(tex_filter_select);
        let tex_dim = m.input("tex_dim", 2);
        front_pipe.aux_input("tex_dim", front_pipe_inner.tex_dim).drive(tex_dim);
        let tex_base = m.input("tex_base", REG_TEXTURE_BASE_BITS);
        front_pipe.aux_input("tex_base", front_pipe_inner.tex_base).drive(tex_base);

        //  Inputs
        front_pipe.in_valid.drive(valid);

        front_pipe.input("in_tile_addr", front_pipe_inner.in_tile_addr).drive(tile_addr);

        front_pipe.input("in_r", front_pipe_inner.in_r).drive(r);
        front_pipe.input("in_g", front_pipe_inner.in_g).drive(g);
        front_pipe.input("in_b", front_pipe_inner.in_b).drive(b);
        front_pipe.input("in_a", front_pipe_inner.in_a).drive(a);

        front_pipe.input("in_w_inverse", front_pipe_inner.in_w_inverse).drive(w_inverse);

        front_pipe.input("in_z", front_pipe_inner.in_z).drive(z);

        front_pipe.input("in_s", front_pipe_inner.in_s).drive(s);
        front_pipe.input("in_t", front_pipe_inner.in_t).drive(t);

        //  Outputs
        let tile_addr = front_pipe.output("out_tile_addr", front_pipe_inner.out_tile_addr);

        let r = front_pipe.output("out_r", front_pipe_inner.out_r);
        let g = front_pipe.output("out_g", front_pipe_inner.out_g);
        let b = front_pipe.output("out_b", front_pipe_inner.out_b);
        let a = front_pipe.output("out_a", front_pipe_inner.out_a);

        let z = front_pipe.output("out_z", front_pipe_inner.out_z);

        let s_fract = front_pipe.output("out_s_fract", front_pipe_inner.out_s_fract);
        let one_minus_s_fract = front_pipe.output("out_one_minus_s_fract", front_pipe_inner.out_one_minus_s_fract);
        let t_fract = front_pipe.output("out_t_fract", front_pipe_inner.out_t_fract);
        let one_minus_t_fract = front_pipe.output("out_one_minus_t_fract", front_pipe_inner.out_one_minus_t_fract);

        for i in 0..4 {
            front_pipe.output(format!("out_tex_buffer{}_read_addr", i), front_pipe_inner.out_tex_buffer_read_addrs[i]);
        }

        depth_test_pipe.out_ready.drive(front_pipe.in_ready | depth_test_reject);

        // TODO: I don't like that these valid signals have moved
        let valid = front_pipe.out_valid.unwrap();

        // Tex cache
        let tex_cache = TexCache::new("tex_cache", m);

        //  Aux
        let tex_cache_invalidate = m.input("tex_cache_invalidate", 1);
        tex_cache.invalidate.drive(tex_cache_invalidate);

        let tex_cache_system_port = tex_cache.system_port.forward("tex_cache_system", m);

        //  Inputs
        front_pipe.out_ready.drive(tex_cache.in_ready);

        tex_cache.in_valid.drive(valid);
        tex_cache.forward_inputs["tile_addr"].drive(tile_addr);

        tex_cache.forward_inputs["r"].drive(r);
        tex_cache.forward_inputs["g"].drive(g);
        tex_cache.forward_inputs["b"].drive(b);
        tex_cache.forward_inputs["a"].drive(a);

        tex_cache.forward_inputs["z"].drive(z);

        tex_cache.forward_inputs["s_fract"].drive(s_fract);
        tex_cache.forward_inputs["one_minus_s_fract"].drive(one_minus_s_fract);
        tex_cache.forward_inputs["t_fract"].drive(t_fract);
        tex_cache.forward_inputs["one_minus_t_fract"].drive(one_minus_t_fract);

        for i in 0..4 {
            let out_tex_buffer_read_addr = front_pipe.output(format!("out_tex_buffer_read_addr_{}", i), front_pipe_inner.out_tex_buffer_read_addrs[i]);
            tex_cache.in_tex_buffer_read_addrs[i].drive(out_tex_buffer_read_addr);
        }

        //  Outputs
        let valid = tex_cache.out_valid;
        let tile_addr = tex_cache.forward_outputs["tile_addr"];

        let r = tex_cache.forward_outputs["r"];
        let g = tex_cache.forward_outputs["g"];
        let b = tex_cache.forward_outputs["b"];
        let a = tex_cache.forward_outputs["a"];

        let z = tex_cache.forward_outputs["z"];

        let s_fract = tex_cache.forward_outputs["s_fract"];
        let one_minus_s_fract = tex_cache.forward_outputs["one_minus_s_fract"];
        let t_fract = tex_cache.forward_outputs["t_fract"];
        let one_minus_t_fract = tex_cache.forward_outputs["one_minus_t_fract"];

        // Back pipe
        let back_pipe = BackPipe::new("back_pipe", m);

        //  Aux
        let depth_write_mask_enable = m.input("depth_write_mask_enable", 1);
        back_pipe.in_depth_write_mask_enable.drive(depth_write_mask_enable);

        let blend_src_factor = m.input("blend_src_factor", REG_BLEND_SETTINGS_SRC_FACTOR_BITS);
        back_pipe.in_blend_src_factor.drive(blend_src_factor);
        let blend_dst_factor = m.input("blend_dst_factor", REG_BLEND_SETTINGS_DST_FACTOR_BITS);
        back_pipe.in_blend_dst_factor.drive(blend_dst_factor);

        let color_buffer_read_port_addr = m.output("color_buffer_read_port_addr", back_pipe.color_buffer_read_port_addr);
        let color_buffer_read_port_enable = m.output("color_buffer_read_port_enable", back_pipe.color_buffer_read_port_enable);

        let color_buffer_read_port_value = m.input("color_buffer_read_port_value", 128);
        back_pipe.color_buffer_read_port_value.drive(color_buffer_read_port_value);

        let color_buffer_write_port_addr = m.output("color_buffer_write_port_addr", back_pipe.color_buffer_write_port_addr);
        let color_buffer_write_port_value = m.output("color_buffer_write_port_value", back_pipe.color_buffer_write_port_value);
        let color_buffer_write_port_enable = m.output("color_buffer_write_port_enable", back_pipe.color_buffer_write_port_enable);
        let color_buffer_write_port_word_enable = m.output("color_buffer_write_port_word_enable", back_pipe.color_buffer_write_port_word_enable);

        let depth_buffer_write_port_addr = m.output("depth_buffer_write_port_addr", back_pipe.depth_buffer_write_port_addr);
        let depth_buffer_write_port_value = m.output("depth_buffer_write_port_value", back_pipe.depth_buffer_write_port_value);
        let depth_buffer_write_port_enable = m.output("depth_buffer_write_port_enable", back_pipe.depth_buffer_write_port_enable);
        let depth_buffer_write_port_word_enable = m.output("depth_buffer_write_port_word_enable", back_pipe.depth_buffer_write_port_word_enable);

        //  Inputs
        back_pipe.in_valid.drive(valid);
        back_pipe.in_tile_addr.drive(tile_addr);

        back_pipe.in_r.drive(r);
        back_pipe.in_g.drive(g);
        back_pipe.in_b.drive(b);
        back_pipe.in_a.drive(a);

        back_pipe.in_z.drive(z);

        back_pipe.in_s_fract.drive(s_fract);
        back_pipe.in_one_minus_s_fract.drive(one_minus_s_fract);
        back_pipe.in_t_fract.drive(t_fract);
        back_pipe.in_one_minus_t_fract.drive(one_minus_t_fract);

        for i in 0..4 {
            back_pipe.in_tex_buffer_read_values[i].drive(tex_cache.out_tex_buffer_read_values[i]);
        }

        //  Outputs
        let valid = back_pipe.out_valid;

        active.drive_next(if_(start, {
            m.high()
        }).else_if(finished_pixel_acc.eq(m.lit(TILE_PIXELS, TILE_PIXELS_BITS + 1)), {
            m.low()
        }).else_({
            active
        }));

        // TODO: There's gotta be a simpler way to do this!
        let finished_pixel_bits = edge_test_reject.concat(depth_test_reject).concat(valid);
        let finished_pixel_count = if_(finished_pixel_bits.eq(m.lit(0b000u32, 3)), {
            m.lit(0u32, 2)
        }).else_if(finished_pixel_bits.eq(m.lit(0b001u32, 3)), {
            m.lit(1u32, 2)
        }).else_if(finished_pixel_bits.eq(m.lit(0b010u32, 3)), {
            m.lit(1u32, 2)
        }).else_if(finished_pixel_bits.eq(m.lit(0b011u32, 3)), {
            m.lit(2u32, 2)
        }).else_if(finished_pixel_bits.eq(m.lit(0b100u32, 3)), {
            m.lit(1u32, 2)
        }).else_if(finished_pixel_bits.eq(m.lit(0b101u32, 3)), {
            m.lit(2u32, 2)
        }).else_if(finished_pixel_bits.eq(m.lit(0b110u32, 3)), {
            m.lit(2u32, 2)
        }).else_({
            m.lit(3u32, 2)
        });

        finished_pixel_acc.drive_next(if_(start, {
            m.lit(0u32, TILE_PIXELS_BITS + 1)
        }).else_({
            finished_pixel_acc + m.lit(0u32, TILE_PIXELS_BITS - 1).concat(finished_pixel_count)
        }));

        PixelPipe {
            m,

            // Control
            start,
            active: m.output("active", active),

            // Inputs
            in_valid,
            in_tile_addr,

            in_w0,
            in_w1,
            in_w2,

            in_r,
            in_g,
            in_b,
            in_a,

            in_w_inverse,

            in_z,

            in_s,
            in_t,

            tex_cache_system_port,

            // Aux inputs
            depth_test_enable,
            depth_buffer_read_port_value,

            tex_filter_select,
            tex_dim,
            tex_base,

            tex_cache_invalidate,

            depth_write_mask_enable,
            blend_src_factor,
            blend_dst_factor,
            color_buffer_read_port_value,

            // Outputs
            in_ready,

            // Aux outputs
            depth_buffer_read_port_addr,
            depth_buffer_read_port_enable,

            color_buffer_read_port_addr,
            color_buffer_read_port_enable,

            color_buffer_write_port_addr,
            color_buffer_write_port_value,
            color_buffer_write_port_enable,
            color_buffer_write_port_word_enable,

            depth_buffer_write_port_addr,
            depth_buffer_write_port_value,
            depth_buffer_write_port_enable,
            depth_buffer_write_port_word_enable,
        }
    }
}

struct DepthTestPipe<'a> {
    #[allow(unused)]
    pub m: &'a Module<'a>,

    // Inputs
    pub in_valid: &'a Input<'a>,
    pub in_tile_addr: &'a Input<'a>,

    pub in_r: &'a Input<'a>,
    pub in_g: &'a Input<'a>,
    pub in_b: &'a Input<'a>,
    pub in_a: &'a Input<'a>,

    pub in_w_inverse: &'a Input<'a>,

    pub in_z: &'a Input<'a>,

    pub in_s: &'a Input<'a>,
    pub in_t: &'a Input<'a>,

    // Aux inputs
    pub depth_test_enable: &'a Input<'a>,

    pub depth_buffer_read_port_value: &'a Input<'a>,

    // Outputs
    pub depth_buffer_read_port_addr: &'a Output<'a>,
    pub depth_buffer_read_port_enable: &'a Output<'a>,

    pub out_valid: &'a Output<'a>,
    pub out_tile_addr: &'a Output<'a>,

    pub out_r: &'a Output<'a>,
    pub out_g: &'a Output<'a>,
    pub out_b: &'a Output<'a>,
    pub out_a: &'a Output<'a>,

    pub out_w_inverse: &'a Output<'a>,

    pub out_z: &'a Output<'a>,

    pub out_s: &'a Output<'a>,
    pub out_t: &'a Output<'a>,

    pub out_depth_test_result: &'a Output<'a>,
}

impl<'a> DepthTestPipe<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> DepthTestPipe<'a> {
        let m = p.module(instance_name, "DepthTestPipe");

        // Inputs
        let in_valid = m.input("in_valid", 1);
        let in_tile_addr = m.input("in_tile_addr", TILE_PIXELS_BITS);

        let in_r = m.input("in_r", COLOR_WHOLE_BITS - 1);
        let in_g = m.input("in_g", COLOR_WHOLE_BITS - 1);
        let in_b = m.input("in_b", COLOR_WHOLE_BITS - 1);
        let in_a = m.input("in_a", COLOR_WHOLE_BITS - 1);

        let in_w_inverse = m.input("in_w_inverse", 32);

        let in_z = m.input("in_z", 16);

        let in_s = m.input("in_s", 32 - RESTORED_W_FRACT_BITS);
        let in_t = m.input("in_t", 32 - RESTORED_W_FRACT_BITS);

        // Aux inputs
        let depth_test_enable = m.input("depth_test_enable", 1);

        let valid = in_valid;
        let tile_addr = in_tile_addr;

        let r = in_r;
        let g = in_g;
        let b = in_b;
        let a = in_a;

        let w_inverse = in_w_inverse;

        let z = in_z;

        let s = in_s;
        let t = in_t;

        //  Issue depth buffer read for prev_depth
        let depth_buffer_read_port_addr = m.output("depth_buffer_read_port_addr", tile_addr.bits(TILE_PIXELS_BITS - 1, 3));
        let depth_buffer_read_port_enable = m.output("depth_buffer_read_port_enable", valid & depth_test_enable);

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
        let depth_buffer_read_port_value = m.input("depth_buffer_read_port_value", 128);
        let prev_depth = depth_buffer_read_port_value;
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
        let out_valid = m.output("out_valid", valid);
        let out_tile_addr = m.output("out_tile_addr", tile_addr);

        let out_r = m.output("out_r", r);
        let out_g = m.output("out_g", g);
        let out_b = m.output("out_b", b);
        let out_a = m.output("out_a", a);

        let out_w_inverse = m.output("out_w_inverse", w_inverse);

        let out_z = m.output("out_z", z);

        let out_s = m.output("out_s", s);
        let out_t = m.output("out_t", t);

        let out_depth_test_result = m.output("out_depth_test_result", depth_test_result);

        DepthTestPipe {
            m,

            // Inputs
            in_valid,
            in_tile_addr,

            in_r,
            in_g,
            in_b,
            in_a,

            in_w_inverse,

            in_z,

            in_s,
            in_t,

            // Aux inputs
            depth_test_enable,

            depth_buffer_read_port_value,

            // Outputs
            depth_buffer_read_port_addr,
            depth_buffer_read_port_enable,

            out_valid,
            out_tile_addr,

            out_r,
            out_g,
            out_b,
            out_a,

            out_w_inverse,

            out_z,

            out_s,
            out_t,

            out_depth_test_result,
        }
    }
}

pub struct FrontPipe<'a> {
    pub m: &'a Module<'a>,

    // Inputs
    pub in_valid: &'a Input<'a>,
    pub in_tile_addr: &'a Input<'a>,

    pub in_r: &'a Input<'a>,
    pub in_g: &'a Input<'a>,
    pub in_b: &'a Input<'a>,
    pub in_a: &'a Input<'a>,

    pub in_w_inverse: &'a Input<'a>,

    pub in_z: &'a Input<'a>,

    pub in_s: &'a Input<'a>,
    pub in_t: &'a Input<'a>,

    // Aux inputs
    pub tex_filter_select: &'a Input<'a>,
    pub tex_dim: &'a Input<'a>,
    pub tex_base: &'a Input<'a>,

    // Outputs
    pub out_valid: &'a Output<'a>,
    pub out_tile_addr: &'a Output<'a>,

    pub out_r: &'a Output<'a>,
    pub out_g: &'a Output<'a>,
    pub out_b: &'a Output<'a>,
    pub out_a: &'a Output<'a>,

    pub out_z: &'a Output<'a>,

    pub out_s_fract: &'a Output<'a>,
    pub out_one_minus_s_fract: &'a Output<'a>,
    pub out_t_fract: &'a Output<'a>,
    pub out_one_minus_t_fract: &'a Output<'a>,

    pub out_tex_buffer_read_addrs: Vec<&'a Output<'a>>,
}

impl<'a> FrontPipe<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> FrontPipe<'a> {
        let m = p.module(instance_name, "FrontPipe");

        // Inputs
        let in_valid = m.input("in_valid", 1);
        let in_tile_addr = m.input("in_tile_addr", TILE_PIXELS_BITS);

        let in_r = m.input("in_r", COLOR_WHOLE_BITS - 1);
        let in_g = m.input("in_g", COLOR_WHOLE_BITS - 1);
        let in_b = m.input("in_b", COLOR_WHOLE_BITS - 1);
        let in_a = m.input("in_a", COLOR_WHOLE_BITS - 1);

        let in_w_inverse = m.input("in_w_inverse", 32);

        let in_z = m.input("in_z", 16);

        let in_s = m.input("in_s", 32 - RESTORED_W_FRACT_BITS);
        let in_t = m.input("in_t", 32 - RESTORED_W_FRACT_BITS);

        // Aux inputs
        let tex_filter_select = m.input("tex_filter_select", 1);
        let tex_dim = m.input("tex_dim", 2);
        let tex_base = m.input("tex_base", REG_TEXTURE_BASE_BITS);

        let mut valid: &dyn Signal<'a> = in_valid;
        let mut tile_addr: &dyn Signal<'a> = in_tile_addr;

        let mut r: &dyn Signal<'a> = in_r;
        let mut g: &dyn Signal<'a> = in_g;
        let mut b: &dyn Signal<'a> = in_b;
        let mut a: &dyn Signal<'a> = in_a;

        let mut z: &dyn Signal<'a> = in_z;

        let mut s: &dyn Signal<'a> = in_s;
        let mut t: &dyn Signal<'a> = in_t;

        let w_approx_reciprocal = ApproxReciprocal::new("WInverseReciprocal", W_INVERSE_FRACT_BITS - RESTORED_W_FRACT_BITS - 3, 4, m);
        w_approx_reciprocal.x.drive(in_w_inverse);

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
        }

        //  Returned from issue before stage 1
        let w = w_approx_reciprocal.quotient;

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

        // Outputs
        let out_valid = m.output("out_valid", valid);
        let out_tile_addr = m.output("out_tile_addr", tile_addr);

        let out_r = m.output("out_r", r);
        let out_g = m.output("out_g", g);
        let out_b = m.output("out_b", b);
        let out_a = m.output("out_a", a);

        let out_z = m.output("out_z", z);

        let out_s_fract = m.output("out_s_fract", s_fract);
        let out_one_minus_s_fract = m.output("out_one_minus_s_fract", one_minus_s_fract);
        let out_t_fract = m.output("out_t_fract", t_fract);
        let out_one_minus_t_fract = m.output("out_one_minus_t_fract", one_minus_t_fract);

        let buffer0_s = (s_floor.bits(6, 0) + m.lit(1u32, 7)).bits(6, 1);
        let buffer0_t = (t_floor.bits(6, 0) + m.lit(1u32, 7)).bits(6, 1);
        let buffer1_s = s_floor.bits(6, 1);
        let buffer1_t = buffer0_t;
        let buffer2_s = buffer0_s;
        let buffer2_t = t_floor.bits(6, 1);
        let buffer3_s = buffer1_s;
        let buffer3_t = buffer2_t;
        let read_addr = |s: &'a dyn Signal<'a>, t: &'a dyn Signal<'a>, buffer_index: u32| {
            let buffer_index = m.lit(buffer_index, 2);
            if_(tex_dim.eq(m.lit(REG_TEXTURE_SETTINGS_DIM_16, REG_TEXTURE_SETTINGS_DIM_BITS)), {
                tex_base
                .bits(REG_TEXTURE_BASE_BITS - 1, 0)
                .concat(buffer_index)
                .concat(t.bits(2, 0))
                .concat(s.bits(2, 0))
            }).else_if(tex_dim.eq(m.lit(REG_TEXTURE_SETTINGS_DIM_32, REG_TEXTURE_SETTINGS_DIM_BITS)), {
                tex_base
                .bits(REG_TEXTURE_BASE_BITS - 1, 2)
                .concat(buffer_index)
                .concat(t.bits(3, 0))
                .concat(s.bits(3, 0))
            }).else_if(tex_dim.eq(m.lit(REG_TEXTURE_SETTINGS_DIM_64, REG_TEXTURE_SETTINGS_DIM_BITS)), {
                tex_base
                .bits(REG_TEXTURE_BASE_BITS - 1, 4)
                .concat(buffer_index)
                .concat(t.bits(4, 0))
                .concat(s.bits(4, 0))
            }).else_({
                // REG_TEXTURE_SETTINGS_DIM_128
                tex_base
                .bits(REG_TEXTURE_BASE_BITS - 1, 6)
                .concat(buffer_index)
                .concat(t)
                .concat(s)
            })
        };
        let out_tex_buffer_read_addrs = vec![
            m.output("out_tex_buffer0_read_addr", read_addr(buffer0_s, buffer0_t, 0)),
            m.output("out_tex_buffer1_read_addr", read_addr(buffer1_s, buffer1_t, 1)),
            m.output("out_tex_buffer2_read_addr", read_addr(buffer2_s, buffer2_t, 2)),
            m.output("out_tex_buffer3_read_addr", read_addr(buffer3_s, buffer3_t, 3)),
        ];

        FrontPipe {
            m,

            // Inputs
            in_valid,
            in_tile_addr,

            in_r,
            in_g,
            in_b,
            in_a,

            in_w_inverse,

            in_z,

            in_s,
            in_t,

            // Aux inputs
            tex_filter_select,
            tex_dim,
            tex_base,

            // Outputs
            out_valid,
            out_tile_addr,

            out_r,
            out_g,
            out_b,
            out_a,

            out_z,

            out_s_fract,
            out_one_minus_s_fract,
            out_t_fract,
            out_one_minus_t_fract,

            out_tex_buffer_read_addrs,
        }
    }
}

pub struct BackPipe<'a> {
    pub m: &'a Module<'a>,

    // Inputs
    in_valid: &'a Input<'a>,
    in_tile_addr: &'a Input<'a>,

    in_r: &'a Input<'a>,
    in_g: &'a Input<'a>,
    in_b: &'a Input<'a>,
    in_a: &'a Input<'a>,

    in_z: &'a Input<'a>,

    in_s_fract: &'a Input<'a>,
    in_one_minus_s_fract: &'a Input<'a>,
    in_t_fract: &'a Input<'a>,
    in_one_minus_t_fract: &'a Input<'a>,

    // Aux inputs
    in_depth_write_mask_enable: &'a Input<'a>,

    in_blend_src_factor: &'a Input<'a>,
    in_blend_dst_factor: &'a Input<'a>,

    in_tex_buffer_read_values: Vec<&'a Input<'a>>,

    color_buffer_read_port_value: &'a Input<'a>,

    // Aux outputs
    color_buffer_read_port_addr: &'a Output<'a>,
    color_buffer_read_port_enable: &'a Output<'a>,

    color_buffer_write_port_addr: &'a Output<'a>,
    color_buffer_write_port_value: &'a Output<'a>,
    color_buffer_write_port_enable: &'a Output<'a>,
    color_buffer_write_port_word_enable: &'a Output<'a>,

    depth_buffer_write_port_addr: &'a Output<'a>,
    depth_buffer_write_port_value: &'a Output<'a>,
    depth_buffer_write_port_enable: &'a Output<'a>,
    depth_buffer_write_port_word_enable: &'a Output<'a>,

    // Outputs
    out_valid: &'a Output<'a>,
}

impl<'a> BackPipe<'a> {
    pub fn new(instance_name: impl Into<String>, p: &'a impl ModuleParent<'a>) -> BackPipe<'a> {
        let m = p.module(instance_name, "BackPipe");

        // Inputs
        let in_valid = m.input("in_valid", 1);
        let in_tile_addr = m.input("in_tile_addr", TILE_PIXELS_BITS);

        let in_r = m.input("in_r", COLOR_WHOLE_BITS - 1);
        let in_g = m.input("in_g", COLOR_WHOLE_BITS - 1);
        let in_b = m.input("in_b", COLOR_WHOLE_BITS - 1);
        let in_a = m.input("in_a", COLOR_WHOLE_BITS - 1);

        let in_z = m.input("in_z", 16);

        let in_s_fract = m.input("in_s_fract", ST_FILTER_FRACT_BITS + 1);
        let in_one_minus_s_fract = m.input("in_one_minus_s_fract", ST_FILTER_FRACT_BITS + 1);
        let in_t_fract = m.input("in_t_fract", ST_FILTER_FRACT_BITS + 1);
        let in_one_minus_t_fract = m.input("in_one_minus_t_fract", ST_FILTER_FRACT_BITS + 1);

        let mut in_tex_buffer_read_values = Vec::new();
        for i in 0..4 {
            in_tex_buffer_read_values.push(m.input(format!("in_tex_buffer{}_read_value", i), 32));
        }

        // Aux inputs
        let in_depth_write_mask_enable = m.input("in_depth_write_mask_enable", 1);

        let in_blend_src_factor = m.input("in_blend_src_factor", REG_BLEND_SETTINGS_SRC_FACTOR_BITS);
        let in_blend_dst_factor = m.input("in_blend_dst_factor", REG_BLEND_SETTINGS_DST_FACTOR_BITS);

        let valid = in_valid;
        let tile_addr = in_tile_addr;

        let r = in_r;
        let g = in_g;
        let b = in_b;
        let a = in_a;

        let z = in_z;

        let s_fract = in_s_fract;
        let one_minus_s_fract = in_one_minus_s_fract;
        let t_fract = in_t_fract;
        let one_minus_t_fract = in_one_minus_t_fract;

        let depth_write_mask_enable = in_depth_write_mask_enable;

        let blend_src_factor = in_blend_src_factor;
        let blend_dst_factor = in_blend_dst_factor;

        // Stage 1
        let valid = valid.reg_next_with_default("stage_1_valid", false);
        let tile_addr = tile_addr.reg_next("stage_1_tile_addr");

        let r = r.reg_next("stage_1_r");
        let g = g.reg_next("stage_1_g");
        let b = b.reg_next("stage_1_b");
        let a = a.reg_next("stage_1_a");

        let z = z.reg_next("stage_1_z");

        let s_fract = s_fract.reg_next("stage_1_s_fract");
        let one_minus_s_fract = one_minus_s_fract.reg_next("stage_1_one_minus_s_fract");
        let t_fract = t_fract.reg_next("stage_1_t_fract");
        let one_minus_t_fract = one_minus_t_fract.reg_next("stage_1_one_minus_t_fract");

        let mut tex_buffer_read_values = Vec::new();
        for i in 0..4 {
            tex_buffer_read_values.push(
                in_tex_buffer_read_values[i].reg_next(format!("stage_1_tex_buffer{}_read_value", i))
            );
        }

        struct Texel<'a> {
            r: &'a dyn Signal<'a>,
            g: &'a dyn Signal<'a>,
            b: &'a dyn Signal<'a>,
            a: &'a dyn Signal<'a>,
        }

        impl<'a> Texel<'a> {
            fn new(texel: &'a dyn Signal<'a>) -> Texel<'a> {
                Texel {
                    r: texel.bits(23, 16),
                    g: texel.bits(15, 8),
                    b: texel.bits(7, 0),
                    a: texel.bits(31, 24),
                }
            }

            fn argb(&self) -> &'a dyn Signal<'a> {
                self.a.concat(self.r).concat(self.g).concat(self.b)
            }
        }

        let texel0 = Texel::new(tex_buffer_read_values[0]);
        let texel1 = Texel::new(tex_buffer_read_values[1]);
        let texel2 = Texel::new(tex_buffer_read_values[2]);
        let texel3 = Texel::new(tex_buffer_read_values[3]);

        fn blend_component<'a>(
            a: &'a dyn Signal<'a>,
            b: &'a dyn Signal<'a>,
            a_fract: &'a dyn Signal<'a>,
            b_fract: &'a dyn Signal<'a>,
        ) -> &'a dyn Signal<'a> {
            (a * a_fract + b * b_fract).bits(8 + ST_FILTER_FRACT_BITS - 1, ST_FILTER_FRACT_BITS)
        }

        fn blend_texels<'a>(
            a: &Texel<'a>,
            b: &Texel<'a>,
            a_fract: &'a dyn Signal<'a>,
            b_fract: &'a dyn Signal<'a>,
        ) -> Texel<'a> {
            let a_fract = a_fract.into();
            let b_fract = b_fract.into();
            Texel {
                r: blend_component(a.r, b.r, a_fract, b_fract),
                g: blend_component(a.g, b.g, a_fract, b_fract),
                b: blend_component(a.b, b.b, a_fract, b_fract),
                a: blend_component(a.a, b.a, a_fract, b_fract),
            }
        }

        let lower = blend_texels(&texel0, &texel1, one_minus_s_fract, s_fract).argb();
        let upper = blend_texels(&texel2, &texel3, one_minus_s_fract, s_fract).argb();

        // Stage 2
        let valid = valid.reg_next_with_default("stage_2_valid", false);
        let tile_addr = tile_addr.reg_next("stage_2_tile_addr");

        let r = r.reg_next("stage_2_r");
        let g = g.reg_next("stage_2_g");
        let b = b.reg_next("stage_2_b");
        let a = a.reg_next("stage_2_a");

        let z = z.reg_next("stage_2_z");

        let t_fract = t_fract.reg_next("stage_2_t_fract");
        let one_minus_t_fract = one_minus_t_fract.reg_next("stage_2_one_minus_t_fract");

        let lower = Texel::new(lower.reg_next("stage_2_lower"));
        let upper = Texel::new(upper.reg_next("stage_2_upper"));

        let texel = blend_texels(&lower, &upper, one_minus_t_fract, t_fract).argb();

        // Stage 3
        let valid = valid.reg_next_with_default("stage_3_valid", false);
        let tile_addr = tile_addr.reg_next("stage_3_tile_addr");

        let r = r.reg_next("stage_3_r");
        let g = g.reg_next("stage_3_g");
        let b = b.reg_next("stage_3_b");
        let a = a.reg_next("stage_3_a");

        let z = z.reg_next("stage_3_z");

        let texel = Texel::new(texel.reg_next("stage_3_texel"));

        let scale_comp = |color_comp: &'a dyn Signal<'a>, texel_comp: &'a dyn Signal<'a>| -> &'a dyn Signal<'a> {
            (color_comp * texel_comp).bits(16, 8)
        };

        let r = scale_comp(r, texel.r);
        let g = scale_comp(g, texel.g);
        let b = scale_comp(b, texel.b);
        let a = scale_comp(a, texel.a);

        //  Issue color buffer read for prev_color
        let color_buffer_read_port_addr = m.output("color_buffer_read_port_addr", tile_addr.bits(TILE_PIXELS_BITS - 1, 2));
        let color_buffer_read_port_enable = m.output("color_buffer_read_port_enable", valid);

        // Stage 4
        let valid = valid.reg_next_with_default("stage_4_valid", false);
        let tile_addr = tile_addr.reg_next("stage_4_tile_addr");

        let r = r.reg_next("stage_4_r");
        let g = g.reg_next("stage_4_g");
        let b = b.reg_next("stage_4_b");
        let a = a.reg_next("stage_4_a");

        let z = z.reg_next("stage_4_z");

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
        let color_buffer_read_port_value = m.input("color_buffer_read_port_value", 128);
        let prev_color = color_buffer_read_port_value;
        let prev_color = if_(tile_addr.bits(1, 0).eq(m.lit(0u32, 2)), {
            prev_color.bits(31, 0)
        }).else_if(tile_addr.bits(1, 0).eq(m.lit(1u32, 2)), {
            prev_color.bits(63, 32)
        }).else_if(tile_addr.bits(1, 0).eq(m.lit(2u32, 2)), {
            prev_color.bits(95, 64)
        }).else_({
            prev_color.bits(127, 96)
        });

        // Stage 5
        let valid = valid.reg_next_with_default("stage_5_valid", false);
        let tile_addr = tile_addr.reg_next("stage_5_tile_addr");

        let r = r.reg_next("stage_5_r");
        let g = g.reg_next("stage_5_g");
        let b = b.reg_next("stage_5_b");
        let a = a.reg_next("stage_5_a");

        let z = z.reg_next("stage_5_z");

        let blend_src_factor = blend_src_factor.reg_next("stage_5_blend_src_factor");
        let blend_dst_factor = blend_dst_factor.reg_next("stage_5_blend_dst_factor");

        let prev_color = prev_color.reg_next("stage_5_prev_color");

        let r = (r * blend_src_factor).bits(17, 8);
        let g = (g * blend_src_factor).bits(17, 8);
        let b = (b * blend_src_factor).bits(17, 8);

        let prev_r = m.lit(0u32, 2).concat((prev_color.bits(23, 16) * blend_dst_factor).bits(16, 9));
        let prev_g = m.lit(0u32, 2).concat((prev_color.bits(15, 8) * blend_dst_factor).bits(16, 9));
        let prev_b = m.lit(0u32, 2).concat((prev_color.bits(7, 0) * blend_dst_factor).bits(16, 9));

        let clamp_comp = |comp: &'a dyn Signal<'a>| -> &'a dyn Signal<'a> {
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

        // Stage 6
        let valid = valid.reg_next_with_default("stage_6_valid", false);
        let tile_addr = tile_addr.reg_next("stage_6_tile_addr");

        let z = z.reg_next("stage_6_z");

        let color = color.reg_next("stage_6_color");

        let color_buffer_write_port_addr = m.output("color_buffer_write_port_addr", tile_addr.bits(TILE_PIXELS_BITS - 1, 2));
        let color_buffer_write_port_value = m.output("color_buffer_write_port_value", color.repeat(4));
        let color_buffer_write_port_enable = m.output("color_buffer_write_port_enable", valid);
        let color_buffer_write_port_word_enable = m.output("color_buffer_write_port_word_enable", (0u32..4).fold(None, |acc, x| {
            let word_enable_bit = tile_addr.bits(1, 0).eq(m.lit(x, 2));
            Some(if let Some(acc) = acc {
                word_enable_bit.concat(acc)
            } else {
                word_enable_bit
            })
        }).unwrap());

        let depth_buffer_write_port_addr = m.output("depth_buffer_write_port_addr", tile_addr.bits(TILE_PIXELS_BITS - 1, 3));
        let depth_buffer_write_port_value = m.output("depth_buffer_write_port_value", z.repeat(8));
        let depth_buffer_write_port_enable = m.output("depth_buffer_write_port_enable", valid & depth_write_mask_enable);
        let depth_buffer_write_port_word_enable = m.output("depth_buffer_write_port_word_enable", (0u32..8).fold(None, |acc, x| {
            let word_enable_bit = tile_addr.bits(2, 0).eq(m.lit(x, 3));
            Some(if let Some(acc) = acc {
                word_enable_bit.concat(acc)
            } else {
                word_enable_bit
            })
        }).unwrap());

        // Outputs
        let out_valid = m.output("out_valid", valid);

        BackPipe {
            m,

            // Inputs
            in_valid,
            in_tile_addr,

            in_r,
            in_g,
            in_b,
            in_a,

            in_z,

            in_s_fract,
            in_one_minus_s_fract,
            in_t_fract,
            in_one_minus_t_fract,

            // Aux inputs
            in_depth_write_mask_enable,

            in_blend_src_factor,
            in_blend_dst_factor,

            in_tex_buffer_read_values,

            color_buffer_read_port_value,

            // Outputs
            out_valid,

            // Aux outputs
            color_buffer_read_port_addr,
            color_buffer_read_port_enable,

            color_buffer_write_port_addr,
            color_buffer_write_port_value,
            color_buffer_write_port_enable,
            color_buffer_write_port_word_enable,

            depth_buffer_write_port_addr,
            depth_buffer_write_port_value,
            depth_buffer_write_port_enable,
            depth_buffer_write_port_word_enable,
        }
    }
}
