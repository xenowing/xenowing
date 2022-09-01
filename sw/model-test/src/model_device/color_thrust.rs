use color_thrust_meta::*;

enum TextureFilter {
    Nearest,
    Bilinear,
}

pub enum TextureDim {
    X16,
    X32,
    X64,
    X128,
}

enum BlendSrcFactor {
    Zero,
    One,
    SrcAlpha,
    OneMinusSrcAlpha,
}

enum BlendDstFactor {
    Zero,
    One,
    SrcAlpha,
    OneMinusSrcAlpha,
}

pub struct ColorThrust {
    color_buffer: Box<[u32]>,
    depth_buffer: Box<[u16]>,

    depth_test_enable: bool,
    depth_write_mask_enable: bool,

    texture_filter: TextureFilter,
    texture_dim: TextureDim,
    texture_base: u32,

    blend_src_factor: BlendSrcFactor,
    blend_dst_factor: BlendDstFactor,

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

impl ColorThrust {
    pub fn new() -> ColorThrust {
        ColorThrust {
            color_buffer: vec![0; TILE_PIXELS as usize].into_boxed_slice(),
            depth_buffer: vec![0; TILE_PIXELS as usize].into_boxed_slice(),

            depth_test_enable: false,
            depth_write_mask_enable: false,

            texture_filter: TextureFilter::Nearest,
            texture_dim: TextureDim::X16,
            texture_base: 0,

            blend_src_factor: BlendSrcFactor::One,
            blend_dst_factor: BlendDstFactor::Zero,

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

    pub fn write_reg(&mut self, addr: u32, data: u32, mem: &[u128]) {
        match addr {
            REG_START_ADDR => self.rasterize_primitive(mem),
            REG_DEPTH_SETTINGS_ADDR => {
                self.depth_test_enable = (data & (1 << REG_DEPTH_TEST_ENABLE_BIT)) != 0;
                self.depth_write_mask_enable = (data & (1 << REG_DEPTH_WRITE_MASK_ENABLE_BIT)) != 0;
            }
            REG_TEXTURE_SETTINGS_ADDR => {
                self.texture_filter = match (data >> REG_TEXTURE_SETTINGS_FILTER_SELECT_BIT_OFFSET) & ((1 << REG_TEXTURE_SETTINGS_FILTER_SELECT_BITS) - 1) {
                    REG_TEXTURE_SETTINGS_FILTER_SELECT_NEAREST => TextureFilter::Nearest,
                    REG_TEXTURE_SETTINGS_FILTER_SELECT_BILINEAR => TextureFilter::Bilinear,
                    _ => unreachable!()
                };
                self.texture_dim = match (data >> REG_TEXTURE_SETTINGS_DIM_BIT_OFFSET) & ((1 << REG_TEXTURE_SETTINGS_DIM_BITS) - 1) {
                    REG_TEXTURE_SETTINGS_DIM_16 => TextureDim::X16,
                    REG_TEXTURE_SETTINGS_DIM_32 => TextureDim::X32,
                    REG_TEXTURE_SETTINGS_DIM_64 => TextureDim::X64,
                    REG_TEXTURE_SETTINGS_DIM_128 => TextureDim::X128,
                    _ => unreachable!()
                };
            }
            REG_TEXTURE_BASE_ADDR => {
                self.texture_base = (data >> (6 + 4)) & ((1 << REG_TEXTURE_BASE_BITS) - 1);
            }
            REG_BLEND_SETTINGS_ADDR => {
                self.blend_src_factor = match (data >> REG_BLEND_SETTINGS_SRC_FACTOR_BIT_OFFSET) & ((1 << REG_BLEND_SETTINGS_SRC_FACTOR_BITS) - 1) {
                    REG_BLEND_SETTINGS_SRC_FACTOR_ZERO => BlendSrcFactor::Zero,
                    REG_BLEND_SETTINGS_SRC_FACTOR_ONE => BlendSrcFactor::One,
                    REG_BLEND_SETTINGS_SRC_FACTOR_SRC_ALPHA => BlendSrcFactor::SrcAlpha,
                    REG_BLEND_SETTINGS_SRC_FACTOR_ONE_MINUS_SRC_ALPHA => BlendSrcFactor::OneMinusSrcAlpha,
                    _ => unreachable!()
                };
                self.blend_dst_factor = match (data >> REG_BLEND_SETTINGS_DST_FACTOR_BIT_OFFSET) & ((1 << REG_BLEND_SETTINGS_DST_FACTOR_BITS) - 1) {
                    REG_BLEND_SETTINGS_DST_FACTOR_ZERO => BlendDstFactor::Zero,
                    REG_BLEND_SETTINGS_DST_FACTOR_ONE => BlendDstFactor::One,
                    REG_BLEND_SETTINGS_DST_FACTOR_SRC_ALPHA => BlendDstFactor::SrcAlpha,
                    REG_BLEND_SETTINGS_DST_FACTOR_ONE_MINUS_SRC_ALPHA => BlendDstFactor::OneMinusSrcAlpha,
                    _ => unreachable!()
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

    pub fn read_reg(&mut self, addr: u32) -> u32 {
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
                }) << REG_TEXTURE_SETTINGS_FILTER_SELECT_BIT_OFFSET
            }
            REG_BLEND_SETTINGS_ADDR => {
                (match self.blend_src_factor {
                    BlendSrcFactor::Zero => REG_BLEND_SETTINGS_SRC_FACTOR_ZERO,
                    BlendSrcFactor::One => REG_BLEND_SETTINGS_SRC_FACTOR_ONE,
                    BlendSrcFactor::SrcAlpha => REG_BLEND_SETTINGS_SRC_FACTOR_SRC_ALPHA,
                    BlendSrcFactor::OneMinusSrcAlpha => REG_BLEND_SETTINGS_SRC_FACTOR_ONE_MINUS_SRC_ALPHA,
                } << REG_BLEND_SETTINGS_SRC_FACTOR_BIT_OFFSET) |
                (match self.blend_dst_factor {
                    BlendDstFactor::Zero => REG_BLEND_SETTINGS_DST_FACTOR_ZERO,
                    BlendDstFactor::One => REG_BLEND_SETTINGS_DST_FACTOR_ONE,
                    BlendDstFactor::SrcAlpha => REG_BLEND_SETTINGS_DST_FACTOR_SRC_ALPHA,
                    BlendDstFactor::OneMinusSrcAlpha => REG_BLEND_SETTINGS_DST_FACTOR_ONE_MINUS_SRC_ALPHA,
                } << REG_BLEND_SETTINGS_DST_FACTOR_BIT_OFFSET)
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

    pub fn write_color_buffer_word(&mut self, addr: u32, data: u128) {
        for i in 0..4 {
            self.color_buffer[(addr * 4 + i) as usize] = (data >> (i * 32)) as _;
        }
    }

    pub fn read_color_buffer_word(&mut self, addr: u32) -> u128 {
        let mut ret = 0;
        for i in 0..4 {
            ret |= (self.color_buffer[(addr * 4 + i) as usize] as u128) << (i * 32);
        }
        ret
    }

    pub fn write_depth_buffer_word(&mut self, addr: u32, data: u128) {
        for i in 0..8 {
            self.depth_buffer[(addr * 8 + i) as usize] = (data >> (i * 16)) as _;
        }
    }

    pub fn read_depth_buffer_word(&mut self, addr: u32) -> u128 {
        let mut ret = 0;
        for i in 0..8 {
            ret |= (self.depth_buffer[(addr * 8 + i) as usize] as u128) << (i * 16);
        }
        ret
    }

    fn rasterize_primitive(&mut self, mem: &[u128]) {
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
                    // Swap weights depending on pixel offsets
                    let (s_fract, one_minus_s_fract) = if (s_floor & 1) == 0 {
                        (s_fract, one_minus_s_fract)
                    } else {
                        (one_minus_s_fract, s_fract)
                    };
                    let (t_fract, one_minus_t_fract) = if (t_floor & 1) == 0 {
                        (t_fract, one_minus_t_fract)
                    } else {
                        (one_minus_t_fract, t_fract)
                    };

                    let buffer0_s = (s_floor.wrapping_add(1) >> 1) & 0x3f;
                    let buffer0_t = (t_floor.wrapping_add(1) >> 1) & 0x3f;
                    let buffer1_s = (s_floor >> 1) & 0x3f;
                    let buffer1_t = buffer0_t;
                    let buffer2_s = buffer0_s;
                    let buffer2_t = (t_floor >> 1) & 0x3f;
                    let buffer3_s = buffer1_s;
                    let buffer3_t = buffer2_t;
                    let texel_color0 = self.fetch_texel(buffer0_s, buffer0_t, 0, mem);
                    let texel_color1 = self.fetch_texel(buffer1_s, buffer1_t, 1, mem);
                    let texel_color2 = self.fetch_texel(buffer2_s, buffer2_t, 2, mem);
                    let texel_color3 = self.fetch_texel(buffer3_s, buffer3_t, 3, mem);
                    let a_r = (texel_color0.0 * one_minus_s_fract + texel_color1.0 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let a_g = (texel_color0.1 * one_minus_s_fract + texel_color1.1 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let a_b = (texel_color0.2 * one_minus_s_fract + texel_color1.2 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let a_a = (texel_color0.3 * one_minus_s_fract + texel_color1.3 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let b_r = (texel_color2.0 * one_minus_s_fract + texel_color3.0 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let b_g = (texel_color2.1 * one_minus_s_fract + texel_color3.1 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let b_b = (texel_color2.2 * one_minus_s_fract + texel_color3.2 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let b_a = (texel_color2.3 * one_minus_s_fract + texel_color3.3 * s_fract) >> ST_FILTER_FRACT_BITS;
                    let texel_r = (a_r * one_minus_t_fract + b_r * t_fract) >> ST_FILTER_FRACT_BITS;
                    let texel_g = (a_g * one_minus_t_fract + b_g * t_fract) >> ST_FILTER_FRACT_BITS;
                    let texel_b = (a_b * one_minus_t_fract + b_b * t_fract) >> ST_FILTER_FRACT_BITS;
                    let texel_a = (a_a * one_minus_t_fract + b_a * t_fract) >> ST_FILTER_FRACT_BITS;

                    fn clamp_comp(comp: u32) -> u32 {
                        if (comp & (1 << (COLOR_WHOLE_BITS - 1))) != 0 {
                            0
                        } else {
                            comp & ((1 << (COLOR_WHOLE_BITS - 2)) - 1)
                        }
                    }

                    let r = clamp_comp(r >> COLOR_FRACT_BITS);
                    let g = clamp_comp(g >> COLOR_FRACT_BITS);
                    let b = clamp_comp(b >> COLOR_FRACT_BITS);
                    let a = clamp_comp(a >> COLOR_FRACT_BITS);

                    let scale_comp = |color_comp: u32, texel_comp: u32| -> u32 {
                        (color_comp * texel_comp) >> 8
                    };

                    let r = scale_comp(r, texel_r);
                    let g = scale_comp(g, texel_g);
                    let b = scale_comp(b, texel_b);
                    let a = scale_comp(a, texel_a);

                    let zero = 0;
                    let one = 1 << 8;

                    let blend_src_factor = match self.blend_src_factor {
                        BlendSrcFactor::Zero => zero,
                        BlendSrcFactor::One => one,
                        BlendSrcFactor::SrcAlpha => a,
                        BlendSrcFactor::OneMinusSrcAlpha => one - a,
                    };

                    let blend_dst_factor = match self.blend_dst_factor {
                        BlendDstFactor::Zero => zero,
                        BlendDstFactor::One => one,
                        BlendDstFactor::SrcAlpha => a,
                        BlendDstFactor::OneMinusSrcAlpha => one - a,
                    };

                    let buffer_index = y as usize * TILE_DIM as usize + x as usize;

                    let prev_color = self.color_buffer[buffer_index];

                    let r = (r * blend_src_factor) >> 8;
                    let g = (g * blend_src_factor) >> 8;
                    let b = (b * blend_src_factor) >> 8;

                    let prev_r = (((prev_color >> 16) & 0xff) * blend_dst_factor) >> 9;
                    let prev_g = (((prev_color >> 8) & 0xff) * blend_dst_factor) >> 9;
                    let prev_b = (((prev_color >> 0) & 0xff) * blend_dst_factor) >> 9;

                    let clamp_comp = |comp: u32| -> u32 {
                        if comp >> 8 == 0 {
                            comp
                        } else {
                            0xff
                        }
                    };

                    let r = clamp_comp(r + prev_r);
                    let g = clamp_comp(g + prev_g);
                    let b = clamp_comp(b + prev_b);
                    let a = clamp_comp(a);

                    let color = (a << 24) | (r << 16) | (g << 8) | b;

                    let z = (z >> (Z_FRACT_BITS - 16)) as u16;
                    let depth_test_result = z < self.depth_buffer[buffer_index] || !self.depth_test_enable;

                    if depth_test_result {
                        self.color_buffer[buffer_index] = color;
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

    fn fetch_texel(&self, s: u32, t: u32, buffer_index: u32, mem: &[u128]) -> (u32, u32, u32, u32) {
        //println!("texture base: 0x{:08x} (byte addr: 0x{:08x})", self.texture_base, self.texture_base << (6 + 4));
        //println!("  s: {}, t: {}, buffer_index: {}", s, t, buffer_index);
        let texel_addr = match self.texture_dim {
            TextureDim::X16 | TextureDim::X32 | TextureDim::X128 => todo!(),
            TextureDim::X64 =>
                ((self.texture_base >> 4) << 12) |
                (buffer_index << 10) |
                ((t & 0x1f) << 5) |
                (s & 0x1f),
        };
        //println!("texel addr:   0x{:08x} (byte addr: 0x{:08x})", texel_addr, texel_addr << 2);
        let word_addr = texel_addr >> 2;
        let word = mem[word_addr as usize];
        let texel = (word >> ((texel_addr & 0x03) * 32)) as u32;
        let texel_red = (texel >> 16) & 0xff;
        let texel_green = (texel >> 8) & 0xff;
        let texel_blue = (texel >> 0) & 0xff;
        let texel_alpha = (texel >> 24) & 0xff;
        (texel_red, texel_green, texel_blue, texel_alpha)
    }
}