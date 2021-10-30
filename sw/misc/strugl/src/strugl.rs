use linalg::*;

use crate::device::*;

use rtl::color_thrust::*;

use std::mem;
use std::rc::Rc;

// TODO: Don't specify this here?
pub const WIDTH: usize = 16 * 8;//320;
pub const HEIGHT: usize = 16 * 8;//240;
pub const PIXELS: usize = WIDTH * HEIGHT;

// TODO: Change this..
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: V4,
    pub color: V4,
    pub tex_coord: V2,
}

pub enum TextureFilter {
    Nearest,
    Bilinear,
}

#[derive(Clone, Copy)]
pub enum TextureDim {
    X16,
    X32,
    X64,
    X128,
}

impl TextureDim {
    pub fn to_u32(&self) -> u32 {
        match *self {
            TextureDim::X16 => 16,
            TextureDim::X32 => 32,
            TextureDim::X64 => 64,
            TextureDim::X128 => 128,
        }
    }
}

pub struct Texture {
    data: Rc<TextureData>,
    filter: TextureFilter,
}

// TODO: Refer to actual memory and properly free when dropped
pub struct TextureData {
    dim: TextureDim,
}

pub enum BlendSrcFactor {
    Zero,
    One,
    SrcAlpha,
    OneMinusSrcAlpha,
}

pub enum BlendDstFactor {
    Zero,
    One,
    SrcAlpha,
    OneMinusSrcAlpha,
}

// TODO: Figure out the best representation without duplicating tons of data!!
//  In particular, all of the deltas are the same for each triangle; only the min values vary
#[derive(Clone, Default)]
struct Triangle {
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

pub struct Context<D: Device> {
    device: D,

    pub back_buffer: Vec<u32>,
    depth_buffer: Vec<u16>,

    // TODO: Don't make these public; expose as some kind of register interface instead
    pub depth_test_enable: bool,
    pub depth_write_mask_enable: bool,

    pub texture: Option<Rc<Texture>>,

    pub blend_src_factor: BlendSrcFactor,
    pub blend_dst_factor: BlendDstFactor,

    pub model_view: M4,
    pub projection: M4,

    assembled_triangles: Vec<Vec<Triangle>>,

    pub estimated_frame_bin_cycles: u64,
    pub estimated_frame_reg_cycles: u64,
    pub estimated_frame_xfer_cycles: u64,
    pub estimated_frame_rasterization_cycles: u64,
}

impl<D: Device> Context<D> {
    pub fn new(device: D) -> Context<D> {
        Context {
            device,

            back_buffer: vec![0; PIXELS],
            depth_buffer: vec![0xffff; PIXELS],

            depth_test_enable: false,
            depth_write_mask_enable: false,

            texture: None,

            blend_src_factor: BlendSrcFactor::One,
            blend_dst_factor: BlendDstFactor::Zero,

            model_view: M4::identity(),
            projection: M4::identity(),

            // TODO: Fixed capacity and splitting drawcalls on overflow
            assembled_triangles: vec![Vec::new(); PIXELS / TILE_PIXELS as usize],

            estimated_frame_bin_cycles: 0,
            estimated_frame_reg_cycles: 0,
            estimated_frame_xfer_cycles: 0,
            estimated_frame_rasterization_cycles: 0,
        }
    }

    pub fn alloc_texture(&mut self, data: Rc<TextureData>, filter: TextureFilter) -> Rc<Texture> {
        Rc::new(Texture {
            data,
            filter,
        })
    }

    // TODO: Expose failure possibility in type signature
    pub fn alloc_texture_data(&mut self, dim: TextureDim, data: &[u32]) -> Rc<TextureData> {
        // TODO: Properly allocate in underlying memory
        let mut addr = 0;
        // Upload data
        //  To support reading a filtered texel in one clock cycle, the texture storage organization is a little tricky.
        //  The main idea is to conceptually group texels into 2x2 blocks. For a bilinear-filtered texel, we need to
        //  perform 4 reads - one from each index in a 2x2 block. When a filtered texel straddles more than one 2x2 block,
        //  we still perform reads from each of the the 4 respective 2x2 block indices, except that we read from 1, 2,
        //  or 4 different blocks, instead of reading all 4 indices from the same block. Thus, we organize texel storage
        //  in a "block-index-major" order, such that all of the texels for a given block index are chunked together. We
        //  match this with our texture cache, which consist of 4 smaller caches; one for each chunk. This allows us to
        //  read from all 4 at once each cycle, which satisfies our bandwidth requirement (when the data is in-cache(s)).
        //  The final detail is that the system bus is 128 bits wide, which corresponds to 4 texels. This means that
        //  when we have a chunk-cache miss, we'd be wasting precious system bus bandwidth if we weren't loading 4 texels
        //  at once. So, each 128-bit word contains a 4x1 group of texels of its corresponding chunk (which, given the
        //  above, corresponds to a span of 8x1 texels in "texture-space", where every 2nd texel is skipped).
        for block_y in 0..2 {
            for block_x in 0..2 {
                for chunk_y in 0..dim.to_u32() / 2 {
                    for chunk_x in 0..dim.to_u32() / 2 / 4 {
                        let mut word = 0;
                        for x in 0..4 {
                            let texel_x = block_x + (chunk_x * 4 + x) * 2;
                            let texel_y = block_y + chunk_y * 2;
                            let argb = data[(texel_y * dim.to_u32() + texel_x) as usize];
                            word |= (argb as u128) << (x * 32);
                        }
                        self.device.write_tex_buffer_word(addr, word);
                        addr += 1;
                    }
                }
            }
        }

        Rc::new(TextureData {
            dim,
        })
    }

    pub fn clear(&mut self) {
        for pixel in &mut self.back_buffer {
            *pixel = 0;
        }
        for depth in &mut self.depth_buffer {
            *depth = 0xffff;
        }

        // TODO: Move?
        self.estimated_frame_bin_cycles = 0;
        self.estimated_frame_reg_cycles = 0;
        self.estimated_frame_xfer_cycles = 0;
        self.estimated_frame_rasterization_cycles = 0;
    }

    pub fn render(&mut self, verts: &mut Vec<Vertex>) {
        // Transformation
        for vert in verts.iter_mut() {
            let object = vert.position;
            let eye = self.model_view * object;
            let clip = self.projection * eye;
            vert.position = clip;
        }

        // Primitive assembly
        for i in (0..verts.len()).step_by(3) {
            self.assemble_triangle([verts[i + 0], verts[i + 1], verts[i + 2]])
        }

        // Per-drawcall rasterizer setup
        self.device.write_reg(
            REG_DEPTH_SETTINGS_ADDR,
            (if self.depth_test_enable { 1 } else { 0 } << REG_DEPTH_TEST_ENABLE_BIT) |
            (if self.depth_write_mask_enable { 1 } else { 0 } << REG_DEPTH_WRITE_MASK_ENABLE_BIT));
        self.estimated_frame_reg_cycles += 1;

        if let Some(texture) = self.texture.as_ref() {
            self.device.write_reg(
                REG_TEXTURE_SETTINGS_ADDR,
                (match texture.filter {
                    TextureFilter::Nearest => REG_TEXTURE_SETTINGS_FILTER_SELECT_NEAREST,
                    TextureFilter::Bilinear => REG_TEXTURE_SETTINGS_FILTER_SELECT_BILINEAR,
                } << REG_TEXTURE_SETTINGS_FILTER_SELECT_BIT_OFFSET) |
                (match texture.data.dim {
                    TextureDim::X16 => REG_TEXTURE_SETTINGS_DIM_16,
                    TextureDim::X32 => REG_TEXTURE_SETTINGS_DIM_32,
                    TextureDim::X64 => REG_TEXTURE_SETTINGS_DIM_64,
                    TextureDim::X128 => REG_TEXTURE_SETTINGS_DIM_128,
                } << REG_TEXTURE_SETTINGS_DIM_BIT_OFFSET));
            self.estimated_frame_reg_cycles += 1;
            // TODO: Proper addr where texture data is loaded
            self.device.write_reg(REG_TEXTURE_BASE_ADDR, 0x00000000);
            self.estimated_frame_reg_cycles += 1;
        }

        self.device.write_reg(
            REG_BLEND_SETTINGS_ADDR,
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
            } << REG_BLEND_SETTINGS_DST_FACTOR_BIT_OFFSET));
        self.estimated_frame_reg_cycles += 1;

        // Primitive rendering
        for tile_index_y in 0..HEIGHT / (TILE_DIM as usize) {
            let tile_min_y = (tile_index_y * (TILE_DIM as usize)) as i32;

            for tile_index_x in 0..WIDTH / (TILE_DIM as usize) {
                let tile_min_x = (tile_index_x * (TILE_DIM as usize)) as i32;

                let tile_index = tile_index_y * (WIDTH / (TILE_DIM as usize)) + tile_index_x;
                let assembled_triangles = &mut self.assembled_triangles[tile_index];
                if assembled_triangles.is_empty() {
                    continue;
                }

                // Copy tile into rasterizer memory
                for y in 0..TILE_DIM as usize {
                    for x in 0..TILE_DIM as usize / 4 {
                        let buffer_index = (HEIGHT - 1 - (tile_min_y as usize + y)) * WIDTH + tile_min_x as usize + x * 4;
                        let mut word = 0;
                        for i in 0..4 {
                            word |= (self.back_buffer[buffer_index + i] as u128) << (i * 32);
                        }
                        self.device.write_color_buffer_word(y as u32 * TILE_DIM / 4 + x as u32, word);

                        self.estimated_frame_xfer_cycles += 1;
                    }
                }
                if self.depth_test_enable || self.depth_write_mask_enable {
                    for y in 0..TILE_DIM as usize {
                        for x in 0..TILE_DIM as usize / 8 {
                            let buffer_index = (HEIGHT - 1 - (tile_min_y as usize + y)) * WIDTH + tile_min_x as usize + x * 8;
                            let mut word = 0;
                            for i in 0..8 {
                                word |= (self.depth_buffer[buffer_index + i] as u128) << (i * 16);
                            }
                            self.device.write_depth_buffer_word(y as u32 * TILE_DIM / 8 + x as u32, word);

                            self.estimated_frame_xfer_cycles += 1;
                        }
                    }
                }

                for triangle in assembled_triangles.iter() {
                    self.device.write_reg(REG_W0_MIN_ADDR, triangle.w0_min);
                    self.device.write_reg(REG_W0_DX_ADDR, triangle.w0_dx);
                    self.device.write_reg(REG_W0_DY_ADDR, triangle.w0_dy);
                    self.device.write_reg(REG_W1_MIN_ADDR, triangle.w1_min);
                    self.device.write_reg(REG_W1_DX_ADDR, triangle.w1_dx);
                    self.device.write_reg(REG_W1_DY_ADDR, triangle.w1_dy);
                    self.device.write_reg(REG_W2_MIN_ADDR, triangle.w2_min);
                    self.device.write_reg(REG_W2_DX_ADDR, triangle.w2_dx);
                    self.device.write_reg(REG_W2_DY_ADDR, triangle.w2_dy);
                    self.device.write_reg(REG_R_MIN_ADDR, triangle.r_min);
                    self.device.write_reg(REG_R_DX_ADDR, triangle.r_dx);
                    self.device.write_reg(REG_R_DY_ADDR, triangle.r_dy);
                    self.device.write_reg(REG_G_MIN_ADDR, triangle.g_min);
                    self.device.write_reg(REG_G_DX_ADDR, triangle.g_dx);
                    self.device.write_reg(REG_G_DY_ADDR, triangle.g_dy);
                    self.device.write_reg(REG_B_MIN_ADDR, triangle.b_min);
                    self.device.write_reg(REG_B_DX_ADDR, triangle.b_dx);
                    self.device.write_reg(REG_B_DY_ADDR, triangle.b_dy);
                    self.device.write_reg(REG_A_MIN_ADDR, triangle.a_min);
                    self.device.write_reg(REG_A_DX_ADDR, triangle.a_dx);
                    self.device.write_reg(REG_A_DY_ADDR, triangle.a_dy);
                    self.device.write_reg(REG_W_INVERSE_MIN_ADDR, triangle.w_inverse_min);
                    self.device.write_reg(REG_W_INVERSE_DX_ADDR, triangle.w_inverse_dx);
                    self.device.write_reg(REG_W_INVERSE_DY_ADDR, triangle.w_inverse_dy);
                    self.device.write_reg(REG_Z_MIN_ADDR, triangle.z_min);
                    self.device.write_reg(REG_Z_DX_ADDR, triangle.z_dx);
                    self.device.write_reg(REG_Z_DY_ADDR, triangle.z_dy);
                    self.device.write_reg(REG_S_MIN_ADDR, triangle.s_min);
                    self.device.write_reg(REG_S_DX_ADDR, triangle.s_dx);
                    self.device.write_reg(REG_S_DY_ADDR, triangle.s_dy);
                    self.device.write_reg(REG_T_MIN_ADDR, triangle.t_min);
                    self.device.write_reg(REG_T_DX_ADDR, triangle.t_dx);
                    self.device.write_reg(REG_T_DY_ADDR, triangle.t_dy);
                    self.estimated_frame_reg_cycles += 33;

                    self.estimated_frame_bin_cycles += mem::size_of::<Triangle>() as u64;

                    // Ensure last primitive is complete
                    while self.device.read_reg(REG_STATUS_ADDR) != 0 {
                        self.estimated_frame_rasterization_cycles += 1;
                    }
                    // Dispatch next primitive
                    self.device.write_reg(REG_START_ADDR, 1);
                }

                // Ensure last primitive is complete
                while self.device.read_reg(REG_STATUS_ADDR) != 0 {
                    self.estimated_frame_rasterization_cycles += 1;
                }

                // Copy rasterizer memory back to tile
                for y in 0..TILE_DIM as usize {
                    for x in 0..TILE_DIM as usize / 4 {
                        let buffer_index = (HEIGHT - 1 - (tile_min_y as usize + y)) * WIDTH + tile_min_x as usize + x * 4;
                        let word = self.device.read_color_buffer_word(y as u32 * TILE_DIM / 4 + x as u32);
                        for i in 0..4 {
                            let tile_pixel = (word >> (32 * i)) as u32;
                            let a = (tile_pixel >> 24) & 0xff;
                            let r = (tile_pixel >> 16) & 0xff;
                            let g = (tile_pixel >> 8) & 0xff;
                            let b = (tile_pixel >> 0) & 0xff;
                            /*let r = r + 16 * (assembled_triangles.len() as u32 - 1);
                            let g = g + 16;
                            let r = if r > 255 { 255 } else { r };
                            let g = if g > 255 { 255 } else { g };*/
                            self.back_buffer[buffer_index + i] = (a << 24) | (r << 16) | (g << 8) | b;
                        }

                        self.estimated_frame_xfer_cycles += 1;
                    }
                }
                if self.depth_write_mask_enable {
                    for y in 0..TILE_DIM as usize {
                        for x in 0..TILE_DIM as usize / 8 {
                            let buffer_index = (HEIGHT - 1 - (tile_min_y as usize + y)) * WIDTH + tile_min_x as usize + x * 8;
                            let word = self.device.read_depth_buffer_word(y as u32 * TILE_DIM / 8 + x as u32);
                            for i in 0..8 {
                                self.depth_buffer[buffer_index + i] = (word >> (16 * i)) as _;
                            }

                            self.estimated_frame_xfer_cycles += 1;
                        }
                    }
                }

                assembled_triangles.clear();
            }
        }
    }

    fn assemble_triangle(&mut self, mut verts: [Vertex; 3]) {
        // TODO: Proper viewport
        let viewport_x = 0;
        let viewport_y = 0;
        let viewport_width = WIDTH;
        let viewport_height = HEIGHT;

        // TODO: Clipping, culling, ...
        for vert in verts.iter() {
            if vert.position.z() < -vert.position.w() || vert.position.z() > vert.position.w() {
                return;
            }
        }

        // Viewport transform
        let mut window_verts = [V3::zero(); 3];
        for i in 0..3 {
            let clip = verts[i].position;
            let ndc = V3::new(clip.x(), clip.y(), clip.z()) / clip.w();
            let viewport_near = 0.0;
            let viewport_far = 1.0;
            let viewport_scale = V3::new(viewport_width as f32 / 2.0, viewport_height as f32 / 2.0, (viewport_far - viewport_near) / 2.0);
            let viewport_bias = V3::new(viewport_x as f32 + viewport_width as f32 / 2.0, viewport_y as f32 + viewport_height as f32 / 2.0, (viewport_far + viewport_near) / 2.0);
            window_verts[i] = ndc * viewport_scale + viewport_bias;
        }

        fn orient2d(a: V2, b: V2, c: V2) -> f32 {
            (b.x() - a.x()) * (c.y() - a.y()) - (b.y() - a.y()) * (c.x() - a.x())
        }

        let /*mut */scaled_area = orient2d(
            V2::new(window_verts[0].x(), window_verts[0].y()),
            V2::new(window_verts[1].x(), window_verts[1].y()),
            V2::new(window_verts[2].x(), window_verts[2].y()));

        // Always cull zero-area triangles
        if scaled_area == 0.0 {
            return;
        }

        // Flip backfacing triangles (TODO: Proper back/front face culling)
        if scaled_area < 0.0 {
            return;
            /*let temp = verts[0];
            verts[0] = verts[1];
            verts[1] = temp;
            let temp = window_verts[0];
            window_verts[0] = window_verts[1];
            window_verts[1] = temp;
            scaled_area = -scaled_area;*/
        }

        let texture_dims = V2::splat(self.texture.as_ref().map(|texture| texture.data.dim.to_u32()).unwrap_or(0) as _);
        // Offset to sample texel centers
        let st_bias = self.texture.as_ref().map(|texture| match texture.filter {
            TextureFilter::Nearest => 0.0,
            TextureFilter::Bilinear => -0.5,
        }).unwrap_or(0.0);
        for vert in verts.iter_mut() {
            vert.tex_coord = (vert.tex_coord * texture_dims + st_bias) / vert.position.w();
        }

        let mut bb_min = V2::new(window_verts[0].x(), window_verts[0].y());
        let mut bb_max = bb_min;
        for i in 1..verts.len() {
            bb_min = bb_min.min(V2::new(window_verts[i].x(), window_verts[i].y()));
            bb_max = bb_max.max(V2::new(window_verts[i].x(), window_verts[i].y()));
        }
        bb_min = bb_min.max(V2::new(viewport_x as f32, viewport_y as f32));
        bb_max = bb_max.min(V2::new((viewport_x + viewport_width as i32 - 1) as f32, (viewport_y + viewport_height as i32 - 1) as f32));
        bb_min = bb_min.max(V2::zero());
        bb_max = bb_max.min(V2::new((WIDTH - 1) as f32, (HEIGHT - 1) as f32));
        let bb_min_x = bb_min.x().floor() as i32;
        let bb_min_y = bb_min.y().floor() as i32;
        let bb_max_x = bb_max.x().ceil() as i32;
        let bb_max_y = bb_max.y().ceil() as i32;

        let mut triangle = Triangle::default();

        fn to_fixed(x: f32, fract_bits: u32) -> i32 {
            let bits = x.to_bits() as i32;
            let exponent = ((bits >> 23) & 0xff) - 127 - 23 + (fract_bits as i32);
            let mut result = (bits & 0x7fffff) | 0x800000;
            if exponent < 0 {
                if exponent > -32 {
                    result >>= -exponent;
                } else {
                    result = 0;
                }
            } else {
                if exponent < 32 {
                    result <<= exponent;
                } else {
                    result = 0x7fffffff;
                }
            }
            if ((bits as u32) & 0x80000000) != 0 {
                result = -result;
            }
            result
        }

        let w0_dx = window_verts[1].y() - window_verts[2].y();
        let w1_dx = window_verts[2].y() - window_verts[0].y();
        let w2_dx = window_verts[0].y() - window_verts[1].y();
        let w0_dy = window_verts[2].x() - window_verts[1].x();
        let w1_dy = window_verts[0].x() - window_verts[2].x();
        let w2_dy = window_verts[1].x() - window_verts[0].x();

        triangle.w0_dx = to_fixed(w0_dx, EDGE_FRACT_BITS) as _;
        triangle.w1_dx = to_fixed(w1_dx, EDGE_FRACT_BITS) as _;
        triangle.w2_dx = to_fixed(w2_dx, EDGE_FRACT_BITS) as _;
        triangle.w0_dy = to_fixed(w0_dy, EDGE_FRACT_BITS) as _;
        triangle.w1_dy = to_fixed(w1_dy, EDGE_FRACT_BITS) as _;
        triangle.w2_dy = to_fixed(w2_dy, EDGE_FRACT_BITS) as _;

        let w0_dx = w0_dx / scaled_area;
        let w1_dx = w1_dx / scaled_area;
        let w2_dx = w2_dx / scaled_area;
        let w0_dy = w0_dy / scaled_area;
        let w1_dy = w1_dy / scaled_area;
        let w2_dy = w2_dy / scaled_area;

        let r_dx = verts[0].color.x() * w0_dx + verts[1].color.x() * w1_dx + verts[2].color.x() * w2_dx;
        let g_dx = verts[0].color.y() * w0_dx + verts[1].color.y() * w1_dx + verts[2].color.y() * w2_dx;
        let b_dx = verts[0].color.z() * w0_dx + verts[1].color.z() * w1_dx + verts[2].color.z() * w2_dx;
        let a_dx = verts[0].color.w() * w0_dx + verts[1].color.w() * w1_dx + verts[2].color.w() * w2_dx;
        let r_dy = verts[0].color.x() * w0_dy + verts[1].color.x() * w1_dy + verts[2].color.x() * w2_dy;
        let g_dy = verts[0].color.y() * w0_dy + verts[1].color.y() * w1_dy + verts[2].color.y() * w2_dy;
        let b_dy = verts[0].color.z() * w0_dy + verts[1].color.z() * w1_dy + verts[2].color.z() * w2_dy;
        let a_dy = verts[0].color.w() * w0_dy + verts[1].color.w() * w1_dy + verts[2].color.w() * w2_dy;
        triangle.r_dx = to_fixed(r_dx, COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1) as _;
        triangle.g_dx = to_fixed(g_dx, COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1) as _;
        triangle.b_dx = to_fixed(b_dx, COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1) as _;
        triangle.a_dx = to_fixed(a_dx, COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1) as _;
        triangle.r_dy = to_fixed(r_dy, COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1) as _;
        triangle.g_dy = to_fixed(g_dy, COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1) as _;
        triangle.b_dy = to_fixed(b_dy, COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1) as _;
        triangle.a_dy = to_fixed(a_dy, COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1) as _;

        let w_inverse_dx = 1.0 / verts[0].position.w() * w0_dx + 1.0 / verts[1].position.w() * w1_dx + 1.0 / verts[2].position.w() * w2_dx;
        let w_inverse_dy = 1.0 / verts[0].position.w() * w0_dy + 1.0 / verts[1].position.w() * w1_dy + 1.0 / verts[2].position.w() * w2_dy;
        triangle.w_inverse_dx = to_fixed(w_inverse_dx, W_INVERSE_FRACT_BITS) as _;
        triangle.w_inverse_dy = to_fixed(w_inverse_dy, W_INVERSE_FRACT_BITS) as _;

        let z_dx = window_verts[0].z() * w0_dx + window_verts[1].z() * w1_dx + window_verts[2].z() * w2_dx;
        let z_dy = window_verts[0].z() * w0_dy + window_verts[1].z() * w1_dy + window_verts[2].z() * w2_dy;
        triangle.z_dx = to_fixed(z_dx, Z_FRACT_BITS) as _;
        triangle.z_dy = to_fixed(z_dy, Z_FRACT_BITS) as _;

        let s_dx = verts[0].tex_coord.x() * w0_dx + verts[1].tex_coord.x() * w1_dx + verts[2].tex_coord.x() * w2_dx;
        let t_dx = verts[0].tex_coord.y() * w0_dx + verts[1].tex_coord.y() * w1_dx + verts[2].tex_coord.y() * w2_dx;
        let s_dy = verts[0].tex_coord.x() * w0_dy + verts[1].tex_coord.x() * w1_dy + verts[2].tex_coord.x() * w2_dy;
        let t_dy = verts[0].tex_coord.y() * w0_dy + verts[1].tex_coord.y() * w1_dy + verts[2].tex_coord.y() * w2_dy;
        triangle.s_dx = to_fixed(s_dx, ST_FRACT_BITS) as _;
        triangle.t_dx = to_fixed(t_dx, ST_FRACT_BITS) as _;
        triangle.s_dy = to_fixed(s_dy, ST_FRACT_BITS) as _;
        triangle.t_dy = to_fixed(t_dy, ST_FRACT_BITS) as _;

        for tile_index_y in 0..HEIGHT / (TILE_DIM as usize) {
            let tile_min_y = (tile_index_y * (TILE_DIM as usize)) as i32;
            let tile_max_y = tile_min_y + (TILE_DIM as usize) as i32 - 1;

            if bb_max_y < tile_min_y || bb_min_y > tile_max_y {
                continue;
            }

            for tile_index_x in 0..WIDTH / (TILE_DIM as usize) {
                let tile_min_x = (tile_index_x * (TILE_DIM as usize)) as i32;
                let tile_max_x = tile_min_x + (TILE_DIM as usize) as i32 - 1;

                if bb_max_x < tile_min_x || bb_min_x > tile_max_x {
                    continue;
                }

                let p = V2::new(tile_min_x as f32, tile_min_y as f32) + 0.5; // Offset to sample pixel centers

                // TODO: Proper top/left fill rule
                let w0_min = orient2d(V2::new(window_verts[1].x(), window_verts[1].y()), V2::new(window_verts[2].x(), window_verts[2].y()), p);
                let w1_min = orient2d(V2::new(window_verts[2].x(), window_verts[2].y()), V2::new(window_verts[0].x(), window_verts[0].y()), p);
                let w2_min = orient2d(V2::new(window_verts[0].x(), window_verts[0].y()), V2::new(window_verts[1].x(), window_verts[1].y()), p);
                triangle.w0_min = to_fixed(w0_min, EDGE_FRACT_BITS) as _;
                triangle.w1_min = to_fixed(w1_min, EDGE_FRACT_BITS) as _;
                triangle.w2_min = to_fixed(w2_min, EDGE_FRACT_BITS) as _;

                let w0_min = w0_min / scaled_area;
                let w1_min = w1_min / scaled_area;
                let w2_min = w2_min / scaled_area;

                let r_min = verts[0].color.x() * w0_min + verts[1].color.x() * w1_min + verts[2].color.x() * w2_min;
                let g_min = verts[0].color.y() * w0_min + verts[1].color.y() * w1_min + verts[2].color.y() * w2_min;
                let b_min = verts[0].color.z() * w0_min + verts[1].color.z() * w1_min + verts[2].color.z() * w2_min;
                let a_min = verts[0].color.w() * w0_min + verts[1].color.w() * w1_min + verts[2].color.w() * w2_min;
                triangle.r_min = to_fixed(r_min, COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1) as _;
                triangle.g_min = to_fixed(g_min, COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1) as _;
                triangle.b_min = to_fixed(b_min, COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1) as _;
                triangle.a_min = to_fixed(a_min, COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 1) as _;

                let w_inverse_min = 1.0 / verts[0].position.w() * w0_min + 1.0 / verts[1].position.w() * w1_min + 1.0 / verts[2].position.w() * w2_min;
                triangle.w_inverse_min = to_fixed(w_inverse_min, W_INVERSE_FRACT_BITS) as _;

                let z_min = window_verts[0].z() * w0_min + window_verts[1].z() * w1_min + window_verts[2].z() * w2_min;
                triangle.z_min = to_fixed(z_min, Z_FRACT_BITS) as _;

                let s_min = verts[0].tex_coord.x() * w0_min + verts[1].tex_coord.x() * w1_min + verts[2].tex_coord.x() * w2_min;
                let t_min = verts[0].tex_coord.y() * w0_min + verts[1].tex_coord.y() * w1_min + verts[2].tex_coord.y() * w2_min;
                triangle.s_min = to_fixed(s_min, ST_FRACT_BITS) as _;
                triangle.t_min = to_fixed(t_min, ST_FRACT_BITS) as _;

                let tile_index = tile_index_y * (WIDTH / (TILE_DIM as usize)) + tile_index_x;
                self.assembled_triangles[tile_index].push(triangle.clone());

                self.estimated_frame_bin_cycles += mem::size_of::<Triangle>() as u64;
            }
        }
    }
}
