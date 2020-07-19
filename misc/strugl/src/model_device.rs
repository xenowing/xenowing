use crate::device::*;
use crate::vec4::*;

use rtl::color_thrust::*;

enum TextureFilter {
    Nearest,
    Bilinear,
}

pub struct ModelDevice {
    color_buffer: [u32; TILE_PIXELS as usize],
    depth_buffer: [u16; TILE_PIXELS as usize],

    tex_buffer0: [u32; TEX_BUFFER_PIXELS as usize],
    tex_buffer1: [u32; TEX_BUFFER_PIXELS as usize],
    tex_buffer2: [u32; TEX_BUFFER_PIXELS as usize],
    tex_buffer3: [u32; TEX_BUFFER_PIXELS as usize],

    depth_test_enable: bool,
    depth_write_mask_enable: bool,

    texture_filter: TextureFilter,

    w0_min: u32,
    w0_dx: u32,
    w0_dy: u32,
    w1_min: u32,
    w1_dx: u32,
    w1_dy: u32,
    w2_min: u32,
    w2_dx: u32,
    w2_dy: u32,
    r_min: u32,
    r_dx: u32,
    r_dy: u32,
    g_min: u32,
    g_dx: u32,
    g_dy: u32,
    b_min: u32,
    b_dx: u32,
    b_dy: u32,
    a_min: u32,
    a_dx: u32,
    a_dy: u32,
    w_inverse_min: u32,
    w_inverse_dx: u32,
    w_inverse_dy: u32,
    z_min: u32,
    z_dx: u32,
    z_dy: u32,
    s_min: u32,
    s_dx: u32,
    s_dy: u32,
    t_min: u32,
    t_dx: u32,
    t_dy: u32,
}

impl ModelDevice {
    pub fn new() -> ModelDevice {
        ModelDevice {
            color_buffer: [0; TILE_PIXELS as usize],
            depth_buffer: [0; TILE_PIXELS as usize],

            tex_buffer0: [0; TEX_BUFFER_PIXELS as usize],
            tex_buffer1: [0; TEX_BUFFER_PIXELS as usize],
            tex_buffer2: [0; TEX_BUFFER_PIXELS as usize],
            tex_buffer3: [0; TEX_BUFFER_PIXELS as usize],

            depth_test_enable: false,
            depth_write_mask_enable: false,

            texture_filter: TextureFilter::Nearest,

            w0_min: 0,
            w0_dx: 0,
            w0_dy: 0,
            w1_min: 0,
            w1_dx: 0,
            w1_dy: 0,
            w2_min: 0,
            w2_dx: 0,
            w2_dy: 0,
            r_min: 0,
            r_dx: 0,
            r_dy: 0,
            g_min: 0,
            g_dx: 0,
            g_dy: 0,
            b_min: 0,
            b_dx: 0,
            b_dy: 0,
            a_min: 0,
            a_dx: 0,
            a_dy: 0,
            w_inverse_min: 0,
            w_inverse_dx: 0,
            w_inverse_dy: 0,
            z_min: 0,
            z_dx: 0,
            z_dy: 0,
            s_min: 0,
            s_dx: 0,
            s_dy: 0,
            t_min: 0,
            t_dx: 0,
            t_dy: 0,
        }
    }

    fn rasterize_primitive(&mut self) {
        let mut w0_row = self.w0_min;
        let mut w1_row = self.w1_min;
        let mut w2_row = self.w2_min;
        let mut r_row = self.r_min;
        let mut g_row = self.g_min;
        let mut b_row = self.b_min;
        let mut a_row = self.a_min;
        let mut w_inverse_row = self.w_inverse_min;
        let mut z_row = self.z_min;
        let mut s_row = self.s_min;
        let mut t_row = self.t_min;

        for y in 0..TILE_DIM {
            let mut w0 = w0_row;
            let mut w1 = w1_row;
            let mut w2 = w2_row;
            let mut r = r_row;
            let mut g = g_row;
            let mut b = b_row;
            let mut a = a_row;
            let mut w_inverse = w_inverse_row;
            let mut z = z_row;
            let mut s = s_row;
            let mut t = t_row;

            for x in 0..TILE_DIM {
                if (w0 | w1 | w2) as i32 >= 0 {
                    let buffer_index = y as usize * TILE_DIM as usize + x as usize;
                    const RESTORED_W_FRACT_BITS: u32 = 8; // Must be less than W_INVERSE_FRACT_BITS and ST_FRACT_BITS

                    fn inverse_approx(x: u32) -> u32 {
                        let shl = x.leading_zeros() & 31;
                        let normalized_x = x << shl;
                        // TODO: Why is 3 the magic number here? Is that dependent on the other constants? Can we determine shr a better way?
                        let shr = (64 - 2 * (W_INVERSE_FRACT_BITS - RESTORED_W_FRACT_BITS - 3) - shl) & 31;

                        let mut e = !normalized_x; // 2's complement approximation
                        let mut q = e;
                        for _ in 0..4 { // TODO: Is this the correct number of steps?
                            q += (((q as u64) * (e as u64)) >> 32) as u32;
                            e = (((e as u64) * (e as u64)) >> 32) as u32;
                        }

                        return (q >> shr) | (1 << (32 - shr));
                    }
                    let w_approx = inverse_approx(w_inverse);

                    /*if x == 0 && y == 0 {
                        /*let one = 1 << W_INVERSE_FRACT_BITS;
                        let w = (one << W_INVERSE_FRACT_BITS) / (w_inverse as i64);
                        let w = (w >> (W_INVERSE_FRACT_BITS - RESTORED_W_FRACT_BITS)) as i32;*/
                        println!("***** w_inverse: 0x{:08x}, w: 0x{:08x}, w_approx: 0x{:08x}, error: {}", w_inverse, w, w_approx, (w_approx as i32) - (w as i32));
                    }*/

                    let w = w_approx;

                    let s = (((s as i32) >> RESTORED_W_FRACT_BITS) * (w as i32)) as u32;
                    let t = (((t as i32) >> RESTORED_W_FRACT_BITS) * (w as i32)) as u32;
                    let s_floor = s >> ST_FRACT_BITS;
                    let t_floor = t >> ST_FRACT_BITS;
                    let mut s_fract = (s >> (ST_FRACT_BITS - ST_FILTER_FRACT_BITS)) & ((1 << ST_FILTER_FRACT_BITS) - 1);
                    let mut t_fract = (t >> (ST_FRACT_BITS - ST_FILTER_FRACT_BITS)) & ((1 << ST_FILTER_FRACT_BITS) - 1);
                    let mut one_minus_s_fract = (1 << ST_FILTER_FRACT_BITS) - s_fract;
                    let mut one_minus_t_fract = (1 << ST_FILTER_FRACT_BITS) - t_fract;
                    match self.texture_filter {
                        TextureFilter::Nearest => {
                            // Lock weights for nearest filtering
                            let zero = 0;
                            let one = 1 << ST_FILTER_FRACT_BITS;
                            s_fract = zero;
                            one_minus_s_fract = one;
                            t_fract = zero;
                            one_minus_t_fract = one;
                        }
                        TextureFilter::Bilinear => (), // Do nothing
                    }
                    let texel_color0 = self.fetch_texel(s_floor + 0, t_floor + 0);
                    let texel_color1 = self.fetch_texel(s_floor + 1, t_floor + 0);
                    let texel_color2 = self.fetch_texel(s_floor + 0, t_floor + 1);
                    let texel_color3 = self.fetch_texel(s_floor + 1, t_floor + 1);
                    let a_red = (texel_color0.0 * one_minus_s_fract + texel_color1.0 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let a_green = (texel_color0.1 * one_minus_s_fract + texel_color1.1 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let a_blue = (texel_color0.2 * one_minus_s_fract + texel_color1.2 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let a_alpha = (texel_color0.3 * one_minus_s_fract + texel_color1.3 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let b_red = (texel_color2.0 * one_minus_s_fract + texel_color3.0 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let b_green = (texel_color2.1 * one_minus_s_fract + texel_color3.1 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let b_blue = (texel_color2.2 * one_minus_s_fract + texel_color3.2 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let b_alpha = (texel_color2.3 * one_minus_s_fract + texel_color3.3 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let texel_red = (a_red * one_minus_t_fract + b_red * t_fract) >> ST_FILTER_FRACT_BITS;
                    let texel_green = (a_green * one_minus_t_fract + b_green * t_fract) >> ST_FILTER_FRACT_BITS;
                    let texel_blue = (a_blue * one_minus_t_fract + b_blue * t_fract) >> ST_FILTER_FRACT_BITS;
                    let texel_alpha = (a_alpha * one_minus_t_fract + b_alpha * t_fract) >> ST_FILTER_FRACT_BITS;

                    let src_color = Vec4::new((r >> 12) as f32, (g >> 12) as f32, (b >> 12) as f32, (a >> 12) as f32) * Vec4::new(texel_red as f32, texel_green as f32, texel_blue as f32, texel_alpha as f32) / 256.0;

                    let color = src_color;

                    let color = color.min(Vec4::splat(255.0));
                    let color_red = color.x().floor() as u32;
                    let color_green = color.y().floor() as u32;
                    let color_blue = color.z().floor() as u32;
                    let color_alpha = color.w().floor() as u32;

                    let z = (z >> (Z_FRACT_BITS - 16)) as u16;
                    let depth_test_result = z < self.depth_buffer[buffer_index] || !self.depth_test_enable;

                    if depth_test_result {
                        self.color_buffer[buffer_index] = (color_alpha << 24) | (color_red << 16) | (color_green << 8) | (color_blue << 0);
                        if self.depth_write_mask_enable {
                            self.depth_buffer[buffer_index] = z;
                        }
                    }
                }

                w0 += self.w0_dx;
                w1 += self.w1_dx;
                w2 += self.w2_dx;
                r += self.r_dx;
                g += self.g_dx;
                b += self.b_dx;
                a += self.a_dx;
                w_inverse += self.w_inverse_dx;
                z += self.z_dx;
                s += self.s_dx;
                t += self.t_dx;
            }

            w0_row += self.w0_dy;
            w1_row += self.w1_dy;
            w2_row += self.w2_dy;
            r_row += self.r_dy;
            g_row += self.g_dy;
            b_row += self.b_dy;
            a_row += self.a_dy;
            w_inverse_row += self.w_inverse_dy;
            z_row += self.z_dy;
            s_row += self.s_dy;
            t_row += self.t_dy;
        }
    }

    fn fetch_texel(&self, s: u32, t: u32) -> (u32, u32, u32, u32) {
        let texture_width = 2 << 3;//self.texture_width_shift;
        let texture_height = 2 << 3;//self.texture_height_shift;
        let s = s as usize & (texture_width - 1);
        let t = t as usize & (texture_height - 1);
        // TODO: Proper fetch from correct address
        let texel = self.tex_buffer0[t / 2 * texture_width / 2 + s / 2];
        let texel_red = (texel >> 16) & 0xff;
        let texel_green = (texel >> 8) & 0xff;
        let texel_blue = (texel >> 0) & 0xff;
        let texel_alpha = (texel >> 24) & 0xff;
        (texel_red, texel_green, texel_blue, texel_alpha)
    }
}

impl Device for ModelDevice {
    fn write_reg(&mut self, addr: u32, data: u32) {
        match addr {
            REG_START_ADDR => self.rasterize_primitive(),
            REG_DEPTH_SETTINGS_ADDR => {
                self.depth_test_enable = (data & (1 << REG_DEPTH_TEST_ENABLE_BIT)) != 0;
                self.depth_write_mask_enable = (data & (1 << REG_DEPTH_WRITE_MASK_ENABLE_BIT)) != 0;
            }
            REG_TEXTURE_SETTINGS_ADDR => {
                self.texture_filter = match (data >> REG_TEXTURE_SETTINGS_FILTER_SELECT_BIT) & 1 {
                    REG_TEXTURE_SETTINGS_FILTER_SELECT_NEAREST => TextureFilter::Nearest,
                    REG_TEXTURE_SETTINGS_FILTER_SELECT_BILINEAR => TextureFilter::Bilinear,
                    _ => unreachable!(),
                };
            }
            REG_W0_MIN_ADDR => { self.w0_min = data; }
            REG_W0_DX_ADDR => { self.w0_dx = data; }
            REG_W0_DY_ADDR => { self.w0_dy = data; }
            REG_W1_MIN_ADDR => { self.w1_min = data; }
            REG_W1_DX_ADDR => { self.w1_dx = data; }
            REG_W1_DY_ADDR => { self.w1_dy = data; }
            REG_W2_MIN_ADDR => { self.w2_min = data; }
            REG_W2_DX_ADDR => { self.w2_dx = data; }
            REG_W2_DY_ADDR => { self.w2_dy = data; }
            REG_R_MIN_ADDR => { self.r_min = data; }
            REG_R_DX_ADDR => { self.r_dx = data; }
            REG_R_DY_ADDR => { self.r_dy = data; }
            REG_G_MIN_ADDR => { self.g_min = data; }
            REG_G_DX_ADDR => { self.g_dx = data; }
            REG_G_DY_ADDR => { self.g_dy = data; }
            REG_B_MIN_ADDR => { self.b_min = data; }
            REG_B_DX_ADDR => { self.b_dx = data; }
            REG_B_DY_ADDR => { self.b_dy = data; }
            REG_A_MIN_ADDR => { self.a_min = data; }
            REG_A_DX_ADDR => { self.a_dx = data; }
            REG_A_DY_ADDR => { self.a_dy = data; }
            REG_W_INVERSE_MIN_ADDR => { self.w_inverse_min = data; }
            REG_W_INVERSE_DX_ADDR => { self.w_inverse_dx = data; }
            REG_W_INVERSE_DY_ADDR => { self.w_inverse_dy = data; }
            REG_Z_MIN_ADDR => { self.z_min = data; }
            REG_Z_DX_ADDR => { self.z_dx = data; }
            REG_Z_DY_ADDR => { self.z_dy = data; }
            REG_S_MIN_ADDR => { self.s_min = data; }
            REG_S_DX_ADDR => { self.s_dx = data; }
            REG_S_DY_ADDR => { self.s_dy = data; }
            REG_T_MIN_ADDR => { self.t_min = data; }
            REG_T_DX_ADDR => { self.t_dx = data; }
            REG_T_DY_ADDR => { self.t_dy = data; }
            _ => panic!("Unrecognized addr: {}", addr)
        }
    }

    fn read_reg(&mut self, addr: u32) -> u32 {
        match addr {
            REG_STATUS_ADDR => 0,
            REG_DEPTH_SETTINGS_ADDR => {
                (if self.depth_test_enable { 1 } else { 0 } << REG_DEPTH_TEST_ENABLE_BIT) |
                (if self.depth_write_mask_enable { 1 } else { 0 } << REG_DEPTH_WRITE_MASK_ENABLE_BIT)
            }
            REG_TEXTURE_SETTINGS_ADDR => {
                (match self.texture_filter {
                    TextureFilter::Nearest => REG_TEXTURE_SETTINGS_FILTER_SELECT_NEAREST,
                    TextureFilter::Bilinear => REG_TEXTURE_SETTINGS_FILTER_SELECT_BILINEAR,
                }) << REG_TEXTURE_SETTINGS_FILTER_SELECT_BIT
            }
            REG_W0_MIN_ADDR => self.w0_min,
            REG_W0_DX_ADDR => self.w0_dx,
            REG_W0_DY_ADDR => self.w0_dy,
            REG_W1_MIN_ADDR => self.w1_min,
            REG_W1_DX_ADDR => self.w1_dx,
            REG_W1_DY_ADDR => self.w1_dy,
            REG_W2_MIN_ADDR => self.w2_min,
            REG_W2_DX_ADDR => self.w2_dx,
            REG_W2_DY_ADDR => self.w2_dy,
            REG_R_MIN_ADDR => self.r_min,
            REG_R_DX_ADDR => self.r_dx,
            REG_R_DY_ADDR => self.r_dy,
            REG_G_MIN_ADDR => self.g_min,
            REG_G_DX_ADDR => self.g_dx,
            REG_G_DY_ADDR => self.g_dy,
            REG_B_MIN_ADDR => self.b_min,
            REG_B_DX_ADDR => self.b_dx,
            REG_B_DY_ADDR => self.b_dy,
            REG_A_MIN_ADDR => self.a_min,
            REG_A_DX_ADDR => self.a_dx,
            REG_A_DY_ADDR => self.a_dy,
            REG_W_INVERSE_MIN_ADDR => self.w_inverse_min,
            REG_W_INVERSE_DX_ADDR => self.w_inverse_dx,
            REG_W_INVERSE_DY_ADDR => self.w_inverse_dy,
            REG_Z_MIN_ADDR => self.z_min,
            REG_Z_DX_ADDR => self.z_dx,
            REG_Z_DY_ADDR => self.z_dy,
            REG_S_MIN_ADDR => self.s_min,
            REG_S_DX_ADDR => self.s_dx,
            REG_S_DY_ADDR => self.s_dy,
            REG_T_MIN_ADDR => self.t_min,
            REG_T_DX_ADDR => self.t_dx,
            REG_T_DY_ADDR => self.t_dy,
            _ => panic!("Unrecognized addr: {}", addr)
        }
    }

    fn write_color_buffer_word(&mut self, addr: u32, data: u128) {
        for i in 0..4 {
            self.color_buffer[(addr * 4 + i) as usize] = (data >> (i * 32)) as _;
        }
    }

    fn read_color_buffer_word(&mut self, addr: u32) -> u128 {
        let mut ret = 0;
        for i in 0..4 {
            ret |= (self.color_buffer[(addr * 4 + i) as usize] as u128) << (i * 32);
        }
        ret
    }

    fn write_depth_buffer_word(&mut self, addr: u32, data: u128) {
        for i in 0..8 {
            self.depth_buffer[(addr * 8 + i) as usize] = (data >> (i * 16)) as _;
        }
    }

    fn read_depth_buffer_word(&mut self, addr: u32) -> u128 {
        let mut ret = 0;
        for i in 0..8 {
            ret |= (self.depth_buffer[(addr * 8 + i) as usize] as u128) << (i * 16);
        }
        ret
    }

    fn write_tex_buffer_word(&mut self, addr: u32, data: u128) {
        for i in 0..4 {
            let word = (data >> (32 * i)) as u32;
            match i {
                0 => { self.tex_buffer0[addr as usize] = word; }
                1 => { self.tex_buffer1[addr as usize] = word; }
                2 => { self.tex_buffer2[addr as usize] = word; }
                3 => { self.tex_buffer3[addr as usize] = word; }
                _ => unreachable!()
            }
        }
    }
}
