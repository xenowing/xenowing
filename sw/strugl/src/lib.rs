#![no_std]

#[macro_use]
extern crate alloc;

use color_thrust_interface::device::*;
use color_thrust_interface::params_and_regs::*;

use env::*;

use linalg::*;

use alloc::rc::Rc;
use alloc::vec::Vec;

use core::fmt::Write;

// TODO: Don't specify this here?
pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;
pub const PIXELS: usize = WIDTH * HEIGHT;

pub const FRACT_BITS: u32 = 16;

// TODO: Change this..
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Iv4<FRACT_BITS>,
    pub color: Iv4<FRACT_BITS>,
    pub tex_coord: Iv2<FRACT_BITS>,
}

// TODO: Evaluate whether we still want/need this, as it's idendical to the above struct
#[derive(Clone, Copy)]
struct TransformedVertex {
    position: Iv4<FRACT_BITS>,
    color: Iv4<FRACT_BITS>,
    tex_coord: Iv2<FRACT_BITS>,
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

    pub model_view: Im4<FRACT_BITS>,
    pub projection: Im4<FRACT_BITS>,

    assembled_triangles: Vec<Vec<Triangle>>,
}

pub struct RenderStats {
    pub vertex_transformation_cycles: u64,
    pub primitive_assembly_and_binning_cycles: u64,
    pub num_nonempty_tiles: u32,
    pub total_tile_xfer_cycles: u64,
    pub total_rasterization_cycles: u64,
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

            model_view: Im4::identity(),
            projection: Im4::identity(),

            // TODO: Fixed capacity and splitting drawcalls on overflow
            assembled_triangles: vec![Vec::new(); PIXELS / TILE_PIXELS as usize],
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
            *pixel = 0x00ff00; // TODO: Change back to 0
        }
        for depth in &mut self.depth_buffer {
            *depth = 0xffff;
        }
    }

    pub fn render<W: Write, E: Environment<W>>(&mut self, verts: &[Vertex], total_primitive_assembly_cycles: &mut u64, total_binning_cycles: &mut u64, env: &E) -> RenderStats {
        // Transformation
        let start_cycles = env.cycles();
        let verts = verts.iter().map(|vert| {
            let object = vert.position;
            let eye = self.model_view * object;
            let clip = self.projection * eye;
            TransformedVertex {
                position: clip,
                color: vert.color,
                tex_coord: vert.tex_coord,
            }
        }).collect::<Vec<_>>();
        let vertex_transformation_cycles = env.cycles().wrapping_sub(start_cycles);

        // Primitive assembly
        let start_cycles = env.cycles();
        for i in (0..verts.len()).step_by(3) {
            self.assemble_triangle([verts[i + 0], verts[i + 1], verts[i + 2]], total_primitive_assembly_cycles, total_binning_cycles, env);
        }
        let primitive_assembly_and_binning_cycles = env.cycles().wrapping_sub(start_cycles);

        // Per-drawcall rasterizer setup
        self.device.write_reg(
            REG_DEPTH_SETTINGS_ADDR,
            (if self.depth_test_enable { 1 } else { 0 } << REG_DEPTH_TEST_ENABLE_BIT) |
            (if self.depth_write_mask_enable { 1 } else { 0 } << REG_DEPTH_WRITE_MASK_ENABLE_BIT));

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
            // TODO: Proper addr where texture data is loaded
            self.device.write_reg(REG_TEXTURE_BASE_ADDR, 0x00000000);
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

        let mut num_nonempty_tiles = 0;
        let mut total_tile_xfer_cycles = 0;
        let mut total_rasterization_cycles = 0;

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

                num_nonempty_tiles += 1;

                // Copy tile into rasterizer memory
                let start_cycles = env.cycles();
                for y in 0..TILE_DIM as usize {
                    for x in 0..TILE_DIM as usize / 4 {
                        let buffer_index = (HEIGHT - 1 - (tile_min_y as usize + y)) * WIDTH + tile_min_x as usize + x * 4;
                        let mut word = 0;
                        for i in 0..4 {
                            word |= (self.back_buffer[buffer_index + i] as u128) << (i * 32);
                        }
                        self.device.write_color_buffer_word(y as u32 * TILE_DIM / 4 + x as u32, word);
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
                        }
                    }
                }
                total_tile_xfer_cycles += env.cycles().wrapping_sub(start_cycles);

                let start_cycles = env.cycles();
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

                    // Ensure previous primitive is complete, if any
                    while self.device.read_reg(REG_STATUS_ADDR) != 0 {
                        // Do nothing
                    }
                    // Dispatch next primitive
                    self.device.write_reg(REG_START_ADDR, 1);
                }

                // Ensure last primitive is complete
                while self.device.read_reg(REG_STATUS_ADDR) != 0 {
                    // Do nothing
                }
                total_rasterization_cycles += env.cycles().wrapping_sub(start_cycles);

                // Copy rasterizer memory back to tile
                let start_cycles = env.cycles();
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
                        }
                    }
                }
                total_tile_xfer_cycles += env.cycles().wrapping_sub(start_cycles);

                assembled_triangles.clear();
            }
        }

        RenderStats {
            vertex_transformation_cycles,
            primitive_assembly_and_binning_cycles,
            num_nonempty_tiles,
            total_tile_xfer_cycles,
            total_rasterization_cycles,
        }
    }

    fn assemble_triangle<W: Write, E: Environment<W>>(&mut self, mut verts: [TransformedVertex; 3], total_primitive_assembly_cycles: &mut u64, total_binning_cycles: &mut u64, env: &E) {
        let start_cycles = env.cycles();

        // TODO: Proper viewport
        let viewport_x = 0;
        let viewport_y = 0;
        let viewport_width = WIDTH as i32;
        let viewport_height = HEIGHT as i32;

        // TODO: Clipping, culling, ...
        for vert in verts.iter() {
            if vert.position.z < -vert.position.w || vert.position.z > vert.position.w {
                return;
            }
        }

        // Viewport transform
        let mut window_verts = [Iv3::zero(); 3];
        for i in 0..3 {
            let clip = verts[i].position;
            // TODO: Don't divide, reciprocal multiply
            let ndc = Iv3::new(clip.x, clip.y, clip.z) / clip.w;
            let viewport_near = 0.0;
            let viewport_far = 1.0;
            let viewport_scale = Iv3::new(
                Fixed::from_raw(viewport_width / 2, 0),
                Fixed::from_raw(viewport_height / 2, 0),
                (viewport_far - viewport_near) / 2.0,
            );
            let viewport_bias = Iv3::new(
                Fixed::from_raw(viewport_x + viewport_width / 2, 0),
                Fixed::from_raw(viewport_y + viewport_height / 2, 0),
                (viewport_far + viewport_near) / 2.0,
            );
            window_verts[i] = ndc * viewport_scale + viewport_bias;
        }

        // Note: Without careful rounding, two triangles incident an edge might calculate slightly
        //  different values for the same edge, due to winding. This error only appears to manifest
        //  in the LSB and doesn't appear to matter when the number of fractional bits for edge
        //  functions in the rasterizer is sufficiently high, but may cause issues later when we
        //  do some more rigorous watertightness testing.
        fn orient2d<const FRACT_BITS: u32>(
            a: Iv2<FRACT_BITS>,
            b: Iv2<FRACT_BITS>,
            c: Iv2<FRACT_BITS>,
        ) -> Fixed<FRACT_BITS> {
            (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
        }

        let /*mut */scaled_area = orient2d(
            Iv2::new(window_verts[0].x, window_verts[0].y),
            Iv2::new(window_verts[1].x, window_verts[1].y),
            Iv2::new(window_verts[2].x, window_verts[2].y));

        // Always cull zero-area triangles
        if scaled_area == 0.0.into() {
            return;
        }

        // Flip backfacing triangles (TODO: Proper back/front face culling)
        if scaled_area < 0.0.into() {
            return;
            /*let temp = verts[0];
            verts[0] = verts[1];
            verts[1] = temp;
            let temp = window_verts[0];
            window_verts[0] = window_verts[1];
            window_verts[1] = temp;
            scaled_area = -scaled_area;*/
        }

        let texture_dims = Iv2::splat(Fixed::from_raw(self.texture.as_ref().map(|texture| texture.data.dim.to_u32()).unwrap_or(0) as _, 0));
        // Offset to sample texel centers
        let st_bias: Fixed<FRACT_BITS> = self.texture.as_ref().map(|texture| match texture.filter {
            TextureFilter::Nearest => Fixed::from(0.0),
            TextureFilter::Bilinear => Fixed::from(-0.5),
        }).unwrap_or(0.0.into());
        for vert in verts.iter_mut() {
            // TODO: Don't divide, reciprocal multiply
            vert.tex_coord = (vert.tex_coord * texture_dims + st_bias) / vert.position.w;
        }

        let mut bb_min = Iv2::new(window_verts[0].x, window_verts[0].y);
        let mut bb_max = bb_min;
        for i in 1..verts.len() {
            bb_min = bb_min.min(Iv2::new(window_verts[i].x, window_verts[i].y));
            bb_max = bb_max.max(Iv2::new(window_verts[i].x, window_verts[i].y));
        }
        bb_min = bb_min.max(Iv2::new(
            Fixed::from_raw(viewport_x, 0),
            Fixed::from_raw(viewport_y, 0),
        ));
        bb_max = bb_max.min(Iv2::new(
            Fixed::from_raw(viewport_x + viewport_width - 1, 0),
            Fixed::from_raw(viewport_y + viewport_height - 1, 0),
        ));
        bb_min = bb_min.max(Iv2::zero());
        bb_max = bb_max.min(Iv2::new(
            Fixed::from_raw(WIDTH as i32 - 1, 0),
            Fixed::from_raw(HEIGHT as i32 - 1, 0),
        ));
        let bb_min_x = bb_min.x.floor().into_raw(0);
        let bb_min_y = bb_min.y.floor().into_raw(0);
        let bb_max_x = bb_max.x.ceil().into_raw(0);
        let bb_max_y = bb_max.y.ceil().into_raw(0);

        let mut triangle = Triangle::default();

        let w0_dx = window_verts[1].y - window_verts[2].y;
        let w1_dx = window_verts[2].y - window_verts[0].y;
        let w2_dx = window_verts[0].y - window_verts[1].y;
        let w0_dy = window_verts[2].x - window_verts[1].x;
        let w1_dy = window_verts[0].x - window_verts[2].x;
        let w2_dy = window_verts[1].x - window_verts[0].x;

        triangle.w0_dx = w0_dx.into_raw(EDGE_FRACT_BITS) as _;
        triangle.w1_dx = w1_dx.into_raw(EDGE_FRACT_BITS) as _;
        triangle.w2_dx = w2_dx.into_raw(EDGE_FRACT_BITS) as _;
        triangle.w0_dy = w0_dy.into_raw(EDGE_FRACT_BITS) as _;
        triangle.w1_dy = w1_dy.into_raw(EDGE_FRACT_BITS) as _;
        triangle.w2_dy = w2_dy.into_raw(EDGE_FRACT_BITS) as _;

        // TODO: Don't divide, reciprocal multiply
        let w_dx = Iv3::new(w0_dx, w1_dx, w2_dx) / scaled_area;
        let w_dy = Iv3::new(w0_dy, w1_dy, w2_dy) / scaled_area;

        let r = Iv3::new(verts[0].color.x, verts[1].color.x, verts[2].color.x);
        let g = Iv3::new(verts[0].color.y, verts[1].color.y, verts[2].color.y);
        let b = Iv3::new(verts[0].color.z, verts[1].color.z, verts[2].color.z);
        let a = Iv3::new(verts[0].color.w, verts[1].color.w, verts[2].color.w);
        let r_dx = r.dot(w_dx);
        let g_dx = g.dot(w_dx);
        let b_dx = b.dot(w_dx);
        let a_dx = a.dot(w_dx);
        let r_dy = r.dot(w_dy);
        let g_dy = g.dot(w_dy);
        let b_dy = b.dot(w_dy);
        let a_dy = a.dot(w_dy);
        triangle.r_dx = r_dx.into_raw(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2) as _;
        triangle.g_dx = g_dx.into_raw(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2) as _;
        triangle.b_dx = b_dx.into_raw(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2) as _;
        triangle.a_dx = a_dx.into_raw(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2) as _;
        triangle.r_dy = r_dy.into_raw(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2) as _;
        triangle.g_dy = g_dy.into_raw(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2) as _;
        triangle.b_dy = b_dy.into_raw(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2) as _;
        triangle.a_dy = a_dy.into_raw(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2) as _;

        // TODO: Don't divide, reciprocal multiply
        let w_inverse = Iv3::new(
            Fixed::from(1.0) / verts[0].position.w,
            Fixed::from(1.0) / verts[1].position.w,
            Fixed::from(1.0) / verts[2].position.w,
        );
        let w_inverse_dx = w_inverse.dot(w_dx);
        let w_inverse_dy = w_inverse.dot(w_dy);
        triangle.w_inverse_dx = w_inverse_dx.into_raw(W_INVERSE_FRACT_BITS) as _;
        triangle.w_inverse_dy = w_inverse_dy.into_raw(W_INVERSE_FRACT_BITS) as _;

        let z = Iv3::new(window_verts[0].z, window_verts[1].z, window_verts[2].z);
        let z_dx = z.dot(w_dx);
        let z_dy = z.dot(w_dy);
        triangle.z_dx = z_dx.into_raw(Z_FRACT_BITS) as _;
        triangle.z_dy = z_dy.into_raw(Z_FRACT_BITS) as _;

        let s = Iv3::new(verts[0].tex_coord.x, verts[1].tex_coord.x, verts[2].tex_coord.x);
        let t = Iv3::new(verts[0].tex_coord.y, verts[1].tex_coord.y, verts[2].tex_coord.y);
        let s_dx = s.dot(w_dx);
        let t_dx = t.dot(w_dx);
        let s_dy = s.dot(w_dy);
        let t_dy = t.dot(w_dy);
        triangle.s_dx = s_dx.into_raw(ST_FRACT_BITS) as _;
        triangle.t_dx = t_dx.into_raw(ST_FRACT_BITS) as _;
        triangle.s_dy = s_dy.into_raw(ST_FRACT_BITS) as _;
        triangle.t_dy = t_dy.into_raw(ST_FRACT_BITS) as _;

        *total_primitive_assembly_cycles += env.cycles().wrapping_sub(start_cycles);

        let start_cycles = env.cycles();

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

                let p = Iv2::new(Fixed::from_raw(tile_min_x, 0), Fixed::from_raw(tile_min_y, 0)) + Fixed::from(0.5); // Offset to sample pixel centers

                let w0_min = orient2d(Iv2::new(window_verts[1].x, window_verts[1].y), Iv2::new(window_verts[2].x, window_verts[2].y), p);
                let w1_min = orient2d(Iv2::new(window_verts[2].x, window_verts[2].y), Iv2::new(window_verts[0].x, window_verts[0].y), p);
                let w2_min = orient2d(Iv2::new(window_verts[0].x, window_verts[0].y), Iv2::new(window_verts[1].x, window_verts[1].y), p);
                fn is_top_left<const FRACT_BITS: u32>(a: Iv2<FRACT_BITS>, b: Iv2<FRACT_BITS>) -> bool {
                    // Top edge
                    a.y == b.y && a.x > b.x ||
                    // Left edge
                    a.y > b.y
                }
                let w0_min_bias = if is_top_left(Iv2::new(window_verts[1].x, window_verts[1].y), Iv2::new(window_verts[2].x, window_verts[2].y)) { 0 } else { -1 };
                let w1_min_bias = if is_top_left(Iv2::new(window_verts[2].x, window_verts[2].y), Iv2::new(window_verts[0].x, window_verts[0].y)) { 0 } else { -1 };
                let w2_min_bias = if is_top_left(Iv2::new(window_verts[0].x, window_verts[0].y), Iv2::new(window_verts[1].x, window_verts[1].y)) { 0 } else { -1 };
                triangle.w0_min = (w0_min.into_raw(EDGE_FRACT_BITS) + w0_min_bias) as _;
                triangle.w1_min = (w1_min.into_raw(EDGE_FRACT_BITS) + w1_min_bias) as _;
                triangle.w2_min = (w2_min.into_raw(EDGE_FRACT_BITS) + w2_min_bias) as _;

                // TODO: Don't divide, reciprocal multiply
                let w_min = Iv3::new(w0_min, w1_min, w2_min) / scaled_area;

                let r_min = r.dot(w_min);
                let g_min = g.dot(w_min);
                let b_min = b.dot(w_min);
                let a_min = a.dot(w_min);
                triangle.r_min = r_min.into_raw(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2) as _;
                triangle.g_min = g_min.into_raw(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2) as _;
                triangle.b_min = b_min.into_raw(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2) as _;
                triangle.a_min = a_min.into_raw(COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2) as _;

                let w_inverse_min = w_inverse.dot(w_min);
                triangle.w_inverse_min = w_inverse_min.into_raw(W_INVERSE_FRACT_BITS) as _;

                let z_min = z.dot(w_min);
                triangle.z_min = z_min.into_raw(Z_FRACT_BITS) as _;

                let s_min = s.dot(w_min);
                let t_min = t.dot(w_min);
                triangle.s_min = s_min.into_raw(ST_FRACT_BITS) as _;
                triangle.t_min = t_min.into_raw(ST_FRACT_BITS) as _;

                let tile_index = tile_index_y * (WIDTH / (TILE_DIM as usize)) + tile_index_x;
                self.assembled_triangles[tile_index].push(triangle.clone());
            }
        }

        *total_binning_cycles += env.cycles().wrapping_sub(start_cycles);
    }
}
