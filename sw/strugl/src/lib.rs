#![no_std]

#[macro_use]
extern crate alloc;

mod bit_pusher;

use bit_pusher::*;

use abstract_device::*;
use abstract_environment::*;

use rtl_meta::color_thrust::*;

use linalg::*;

use alloc::rc::Rc;
use alloc::vec::Vec;

use core::fmt::Write;

// TODO: Don't specify this here?
pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;
pub const PIXELS: u32 = WIDTH * HEIGHT;

const DEFAULT_FRACT_BITS: u32 = 16;

const NUM_COLOR_BUFFER_WORDS: u32 = PIXELS * 4 / 16;
const NUM_DEPTH_BUFFER_WORDS: u32 = PIXELS * 2 / 16;

// TODO: Change this..
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Iv4<DEFAULT_FRACT_BITS>,
    pub color: Iv4<DEFAULT_FRACT_BITS>,
    pub tex_coord: Iv2<DEFAULT_FRACT_BITS>,
}

// TODO: Evaluate whether we still want/need this, as it's idendical to the above struct
#[derive(Clone, Copy)]
struct TransformedVertex {
    position: Iv4<DEFAULT_FRACT_BITS>,
    color: Iv4<DEFAULT_FRACT_BITS>,
    tex_coord: Iv2<DEFAULT_FRACT_BITS>,
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

// TODO: Properly free memory when dropped
pub struct TextureData {
    base_addr: u32,
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

    back_buffer_base_addr: u32,
    depth_buffer_base_addr: u32,

    // TODO: Don't make these public; expose as some kind of register interface instead
    pub depth_test_enable: bool,
    pub depth_write_mask_enable: bool,

    pub texture: Option<Rc<Texture>>,

    pub blend_src_factor: BlendSrcFactor,
    pub blend_dst_factor: BlendDstFactor,

    pub model_view: Im4<DEFAULT_FRACT_BITS>,
    pub projection: Im4<DEFAULT_FRACT_BITS>,

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
    pub fn new(mut device: D) -> Context<D> {
        let back_buffer_base_addr = device.mem_alloc(NUM_COLOR_BUFFER_WORDS, 1);
        let depth_buffer_base_addr = device.mem_alloc(NUM_DEPTH_BUFFER_WORDS, 1);

        Context {
            device,

            back_buffer_base_addr,
            depth_buffer_base_addr,

            depth_test_enable: false,
            depth_write_mask_enable: false,

            texture: None,

            blend_src_factor: BlendSrcFactor::One,
            blend_dst_factor: BlendDstFactor::Zero,

            model_view: Im4::identity(),
            projection: Im4::identity(),

            // TODO: Fixed capacity and splitting drawcalls on overflow
            assembled_triangles: vec![Vec::new(); (PIXELS / TILE_PIXELS) as usize],
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
        let align_bits = 6 + match dim {
            TextureDim::X16 => 0,
            TextureDim::X32 => 2,
            TextureDim::X64 => 4,
            TextureDim::X128 => 8,
        };
        let base_addr = self.device.mem_alloc(data.len() as u32 / 4, 1 << align_bits);
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
        // TODO: Non-linear swizzling for better hit rate (be sure to measure/compare first!)
        let mut addr = base_addr;
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
                        self.device.mem_write_word(addr, word);
                        addr += 16;
                    }
                }
            }
        }

        Rc::new(TextureData {
            base_addr,
            dim,
        })
    }

    pub fn clear(&mut self) {
        let clear_color = 0x00ff00; // TODO: Expose this properly
        let mut clear_color_word = 0;
        for i in 0..4 {
            clear_color_word |= (clear_color as u128) << (i * 32);
        }
        for i in 0..NUM_COLOR_BUFFER_WORDS {
            self.device.mem_write_word(self.back_buffer_base_addr + i * 16, clear_color_word);
        }
        let clear_depth = 0xffff; // TODO: Expose this properly
        let mut clear_depth_word = 0;
        for i in 0..8 {
            clear_depth_word |= (clear_depth as u128) << (i * 16);
        }
        for i in 0..NUM_DEPTH_BUFFER_WORDS {
            self.device.mem_write_word(self.depth_buffer_base_addr + i * 16, clear_depth_word);
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
        self.device.color_thrust_write_reg(
            REG_DEPTH_SETTINGS_ADDR,
            (if self.depth_test_enable { 1 } else { 0 } << REG_DEPTH_TEST_ENABLE_BIT) |
            (if self.depth_write_mask_enable { 1 } else { 0 } << REG_DEPTH_WRITE_MASK_ENABLE_BIT));

        if let Some(texture) = self.texture.as_ref() {
            self.device.color_thrust_write_reg(
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
            self.device.color_thrust_write_reg(REG_TEXTURE_BASE_ADDR, texture.data.base_addr);
        }

        self.device.color_thrust_write_reg(
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
        for tile_index_y in 0..HEIGHT / TILE_DIM {
            let tile_min_y = tile_index_y * TILE_DIM;

            for tile_index_x in 0..WIDTH / TILE_DIM {
                let tile_min_x = tile_index_x * TILE_DIM;

                let tile_index = tile_index_y * (WIDTH / TILE_DIM) + tile_index_x;
                let assembled_triangles = &mut self.assembled_triangles[tile_index as usize];
                if assembled_triangles.is_empty() {
                    continue;
                }

                num_nonempty_tiles += 1;

                // Copy tile into rasterizer memory
                let start_cycles = env.cycles();
                mem2sys(
                    &mut self.device,
                    0x04000000, // TODO: Proper constant!!!
                    0,
                    0,
                    self.back_buffer_base_addr + ((HEIGHT - 1 - tile_min_y) * WIDTH + tile_min_x) * 4,
                    TILE_DIM / 4,
                    (-(WIDTH as i32) / 4) as _,
                    TILE_DIM * TILE_DIM / 4,
                );
                if self.depth_test_enable || self.depth_write_mask_enable {
                    mem2sys(
                        &mut self.device,
                        0x05000000, // TODO: Proper constant!!!
                        0,
                        0,
                        self.depth_buffer_base_addr + ((HEIGHT - 1 - tile_min_y) * WIDTH + tile_min_x) * 2,
                        TILE_DIM / 8,
                        (-(WIDTH as i32) / 8) as _,
                        TILE_DIM * TILE_DIM / 8,
                    );
                }
                total_tile_xfer_cycles += env.cycles().wrapping_sub(start_cycles);

                let start_cycles = env.cycles();
                for triangle in assembled_triangles.iter() {
                    self.device.color_thrust_write_reg(REG_W0_MIN_ADDR, triangle.w0_min);
                    self.device.color_thrust_write_reg(REG_W0_DX_ADDR, triangle.w0_dx);
                    self.device.color_thrust_write_reg(REG_W0_DY_ADDR, triangle.w0_dy);
                    self.device.color_thrust_write_reg(REG_W1_MIN_ADDR, triangle.w1_min);
                    self.device.color_thrust_write_reg(REG_W1_DX_ADDR, triangle.w1_dx);
                    self.device.color_thrust_write_reg(REG_W1_DY_ADDR, triangle.w1_dy);
                    self.device.color_thrust_write_reg(REG_W2_MIN_ADDR, triangle.w2_min);
                    self.device.color_thrust_write_reg(REG_W2_DX_ADDR, triangle.w2_dx);
                    self.device.color_thrust_write_reg(REG_W2_DY_ADDR, triangle.w2_dy);
                    self.device.color_thrust_write_reg(REG_R_MIN_ADDR, triangle.r_min);
                    self.device.color_thrust_write_reg(REG_R_DX_ADDR, triangle.r_dx);
                    self.device.color_thrust_write_reg(REG_R_DY_ADDR, triangle.r_dy);
                    self.device.color_thrust_write_reg(REG_G_MIN_ADDR, triangle.g_min);
                    self.device.color_thrust_write_reg(REG_G_DX_ADDR, triangle.g_dx);
                    self.device.color_thrust_write_reg(REG_G_DY_ADDR, triangle.g_dy);
                    self.device.color_thrust_write_reg(REG_B_MIN_ADDR, triangle.b_min);
                    self.device.color_thrust_write_reg(REG_B_DX_ADDR, triangle.b_dx);
                    self.device.color_thrust_write_reg(REG_B_DY_ADDR, triangle.b_dy);
                    self.device.color_thrust_write_reg(REG_A_MIN_ADDR, triangle.a_min);
                    self.device.color_thrust_write_reg(REG_A_DX_ADDR, triangle.a_dx);
                    self.device.color_thrust_write_reg(REG_A_DY_ADDR, triangle.a_dy);
                    self.device.color_thrust_write_reg(REG_W_INVERSE_MIN_ADDR, triangle.w_inverse_min);
                    self.device.color_thrust_write_reg(REG_W_INVERSE_DX_ADDR, triangle.w_inverse_dx);
                    self.device.color_thrust_write_reg(REG_W_INVERSE_DY_ADDR, triangle.w_inverse_dy);
                    self.device.color_thrust_write_reg(REG_Z_MIN_ADDR, triangle.z_min);
                    self.device.color_thrust_write_reg(REG_Z_DX_ADDR, triangle.z_dx);
                    self.device.color_thrust_write_reg(REG_Z_DY_ADDR, triangle.z_dy);
                    self.device.color_thrust_write_reg(REG_S_MIN_ADDR, triangle.s_min);
                    self.device.color_thrust_write_reg(REG_S_DX_ADDR, triangle.s_dx);
                    self.device.color_thrust_write_reg(REG_S_DY_ADDR, triangle.s_dy);
                    self.device.color_thrust_write_reg(REG_T_MIN_ADDR, triangle.t_min);
                    self.device.color_thrust_write_reg(REG_T_DX_ADDR, triangle.t_dx);
                    self.device.color_thrust_write_reg(REG_T_DY_ADDR, triangle.t_dy);

                    // Ensure previous primitive is complete, if any
                    while self.device.color_thrust_read_reg(REG_STATUS_ADDR) != 0 {
                        // Do nothing
                    }
                    // Dispatch next primitive
                    self.device.color_thrust_write_reg(REG_START_ADDR, 1);
                }

                // Ensure last primitive is complete
                while self.device.color_thrust_read_reg(REG_STATUS_ADDR) != 0 {
                    // Do nothing
                }
                total_rasterization_cycles += env.cycles().wrapping_sub(start_cycles);

                // Copy rasterizer memory back to tile
                let start_cycles = env.cycles();
                sys2mem(
                    &mut self.device,
                    0x04000000, // TODO: Proper constant!!!
                    0,
                    0,
                    self.back_buffer_base_addr + ((HEIGHT - 1 - tile_min_y) * WIDTH + tile_min_x) * 4,
                    TILE_DIM / 4,
                    (-(WIDTH as i32) / 4) as _,
                    TILE_DIM * TILE_DIM / 4,
                );
                if self.depth_write_mask_enable {
                    sys2mem(
                        &mut self.device,
                        0x05000000, // TODO: Proper constant!!!
                        0,
                        0,
                        self.depth_buffer_base_addr + ((HEIGHT - 1 - tile_min_y) * WIDTH + tile_min_x) * 2,
                        TILE_DIM / 8,
                        (-(WIDTH as i32) / 8) as _,
                        TILE_DIM * TILE_DIM / 8,
                    );
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
        #[derive(Clone, Copy)]
        struct WindowVert {
            x: Fixed<EDGE_FRACT_BITS>,
            y: Fixed<EDGE_FRACT_BITS>,
            z: Fixed<DEFAULT_FRACT_BITS>,
        }

        let mut window_verts = [WindowVert {
            x: Fixed::zero(),
            y: Fixed::zero(),
            z: Fixed::zero(),
        }; 3];
        for i in 0..3 {
            let clip = verts[i].position;
            // TODO: Don't divide, reciprocal multiply
            let ndc = Iv3::new(clip.x, clip.y, clip.z) / clip.w;
            let viewport_near = 0.0;
            let viewport_far = 1.0;
            window_verts[i].x = ndc.x.mul_mixed(Fixed::<EDGE_FRACT_BITS>::from_raw(viewport_width / 2, 0)) + Fixed::from_raw(viewport_x + viewport_width / 2, 0);
            window_verts[i].y = ndc.y.mul_mixed(Fixed::<EDGE_FRACT_BITS>::from_raw(viewport_height / 2, 0)) + Fixed::from_raw(viewport_y + viewport_height / 2, 0);
            window_verts[i].z = ndc.z * ((viewport_far - viewport_near) / 2.0).into() + ((viewport_far + viewport_near) / 2.0).into();
        }

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
        let st_bias: Fixed<DEFAULT_FRACT_BITS> = self.texture.as_ref().map(|texture| match texture.filter {
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
        let w_dx: Iv3<DEFAULT_FRACT_BITS> = Iv3::new(w0_dx, w1_dx, w2_dx).div_mixed(scaled_area);
        let w_dy: Iv3<DEFAULT_FRACT_BITS> = Iv3::new(w0_dy, w1_dy, w2_dy).div_mixed(scaled_area);

        let r = Iv3::new(verts[0].color.x, verts[1].color.x, verts[2].color.x);
        let g = Iv3::new(verts[0].color.y, verts[1].color.y, verts[2].color.y);
        let b = Iv3::new(verts[0].color.z, verts[1].color.z, verts[2].color.z);
        let a = Iv3::new(verts[0].color.w, verts[1].color.w, verts[2].color.w);
        // TODO: Move this?
        const COLOR_COMP_BITS: u32 = COLOR_WHOLE_BITS + COLOR_FRACT_BITS - 2;
        let r_dx: Fixed<COLOR_COMP_BITS> = r.dot_mixed(w_dx);
        let g_dx: Fixed<COLOR_COMP_BITS> = g.dot_mixed(w_dx);
        let b_dx: Fixed<COLOR_COMP_BITS> = b.dot_mixed(w_dx);
        let a_dx: Fixed<COLOR_COMP_BITS> = a.dot_mixed(w_dx);
        let r_dy: Fixed<COLOR_COMP_BITS> = r.dot_mixed(w_dy);
        let g_dy: Fixed<COLOR_COMP_BITS> = g.dot_mixed(w_dy);
        let b_dy: Fixed<COLOR_COMP_BITS> = b.dot_mixed(w_dy);
        let a_dy: Fixed<COLOR_COMP_BITS> = a.dot_mixed(w_dy);
        triangle.r_dx = r_dx.into_raw(COLOR_COMP_BITS) as _;
        triangle.g_dx = g_dx.into_raw(COLOR_COMP_BITS) as _;
        triangle.b_dx = b_dx.into_raw(COLOR_COMP_BITS) as _;
        triangle.a_dx = a_dx.into_raw(COLOR_COMP_BITS) as _;
        triangle.r_dy = r_dy.into_raw(COLOR_COMP_BITS) as _;
        triangle.g_dy = g_dy.into_raw(COLOR_COMP_BITS) as _;
        triangle.b_dy = b_dy.into_raw(COLOR_COMP_BITS) as _;
        triangle.a_dy = a_dy.into_raw(COLOR_COMP_BITS) as _;

        // TODO: Don't divide, reciprocal multiply
        let w_inverse = Iv3::new(
            Fixed::from(1.0) / verts[0].position.w,
            Fixed::from(1.0) / verts[1].position.w,
            Fixed::from(1.0) / verts[2].position.w,
        );
        let w_inverse_dx: Fixed<W_INVERSE_FRACT_BITS> = w_inverse.dot_mixed(w_dx);
        let w_inverse_dy: Fixed<W_INVERSE_FRACT_BITS> = w_inverse.dot_mixed(w_dy);
        triangle.w_inverse_dx = w_inverse_dx.into_raw(W_INVERSE_FRACT_BITS) as _;
        triangle.w_inverse_dy = w_inverse_dy.into_raw(W_INVERSE_FRACT_BITS) as _;

        let z = Iv3::new(window_verts[0].z, window_verts[1].z, window_verts[2].z);
        let z_dx: Fixed<Z_FRACT_BITS> = z.dot_mixed(w_dx);
        let z_dy: Fixed<Z_FRACT_BITS> = z.dot_mixed(w_dy);
        triangle.z_dx = z_dx.into_raw(Z_FRACT_BITS) as _;
        triangle.z_dy = z_dy.into_raw(Z_FRACT_BITS) as _;

        let s = Iv3::new(verts[0].tex_coord.x, verts[1].tex_coord.x, verts[2].tex_coord.x);
        let t = Iv3::new(verts[0].tex_coord.y, verts[1].tex_coord.y, verts[2].tex_coord.y);
        let s_dx: Fixed<ST_FRACT_BITS> = s.dot_mixed(w_dx);
        let t_dx: Fixed<ST_FRACT_BITS> = t.dot_mixed(w_dx);
        let s_dy: Fixed<ST_FRACT_BITS> = s.dot_mixed(w_dy);
        let t_dy: Fixed<ST_FRACT_BITS> = t.dot_mixed(w_dy);
        triangle.s_dx = s_dx.into_raw(ST_FRACT_BITS) as _;
        triangle.t_dx = t_dx.into_raw(ST_FRACT_BITS) as _;
        triangle.s_dy = s_dy.into_raw(ST_FRACT_BITS) as _;
        triangle.t_dy = t_dy.into_raw(ST_FRACT_BITS) as _;

        fn is_top_left<const FRACT_BITS: u32>(a: Iv2<FRACT_BITS>, b: Iv2<FRACT_BITS>) -> bool {
            // Top edge
            a.y == b.y && a.x > b.x ||
            // Left edge
            a.y > b.y
        }
        let w0_min_bias = if is_top_left(Iv2::new(window_verts[1].x, window_verts[1].y), Iv2::new(window_verts[2].x, window_verts[2].y)) { 0 } else { -1 };
        let w1_min_bias = if is_top_left(Iv2::new(window_verts[2].x, window_verts[2].y), Iv2::new(window_verts[0].x, window_verts[0].y)) { 0 } else { -1 };
        let w2_min_bias = if is_top_left(Iv2::new(window_verts[0].x, window_verts[0].y), Iv2::new(window_verts[1].x, window_verts[1].y)) { 0 } else { -1 };

        *total_primitive_assembly_cycles += env.cycles().wrapping_sub(start_cycles);

        let start_cycles = env.cycles();

        for tile_index_y in 0..HEIGHT / TILE_DIM {
            let tile_min_y = (tile_index_y * TILE_DIM) as i32;
            let tile_max_y = tile_min_y + TILE_DIM as i32 - 1;

            if bb_max_y < tile_min_y || bb_min_y > tile_max_y {
                continue;
            }

            for tile_index_x in 0..WIDTH / TILE_DIM {
                let tile_min_x = (tile_index_x * TILE_DIM) as i32;
                let tile_max_x = tile_min_x + TILE_DIM as i32 - 1;

                if bb_max_x < tile_min_x || bb_min_x > tile_max_x {
                    continue;
                }

                let p = Iv2::new(
                    Fixed::from_raw(tile_min_x, 0),
                    Fixed::from_raw(tile_min_y, 0),
                ) + Fixed::from(0.5); // Offset to sample pixel centers

                // For edge functions, we must be extra careful that we calculate the same values, regardless of edge orientation.
                //  In particular, we must apply the fill-rule bias _before_ we shift out the extra fractional bits from the
                //  multiplications in the determinant calculations. So, we do some of the math here with raw integers and take
                //  extra care when packing them back up (especially for use with interpolants, where we don't want the fill rule
                //  bias to be applied).
                fn orient2d_raw<const FRACT_BITS: u32>(
                    a: Iv2<FRACT_BITS>,
                    b: Iv2<FRACT_BITS>,
                    c: Iv2<FRACT_BITS>,
                ) -> i64 {
                    (b.x.into_raw(FRACT_BITS) - a.x.into_raw(FRACT_BITS)) as i64 *
                        (c.y.into_raw(FRACT_BITS) - a.y.into_raw(FRACT_BITS)) as i64 -
                        (b.y.into_raw(FRACT_BITS) - a.y.into_raw(FRACT_BITS)) as i64 *
                        (c.x.into_raw(FRACT_BITS) - a.x.into_raw(FRACT_BITS)) as i64
                }

                let w0_min_raw = orient2d_raw(Iv2::new(window_verts[1].x, window_verts[1].y), Iv2::new(window_verts[2].x, window_verts[2].y), p);
                let w1_min_raw = orient2d_raw(Iv2::new(window_verts[2].x, window_verts[2].y), Iv2::new(window_verts[0].x, window_verts[0].y), p);
                let w2_min_raw = orient2d_raw(Iv2::new(window_verts[0].x, window_verts[0].y), Iv2::new(window_verts[1].x, window_verts[1].y), p);
                triangle.w0_min = ((w0_min_raw + w0_min_bias) >> EDGE_FRACT_BITS) as _;
                triangle.w1_min = ((w1_min_raw + w1_min_bias) >> EDGE_FRACT_BITS) as _;
                triangle.w2_min = ((w2_min_raw + w2_min_bias) >> EDGE_FRACT_BITS) as _;

                let w0_min: Fixed<EDGE_FRACT_BITS> = Fixed::from_raw((w0_min_raw >> EDGE_FRACT_BITS) as _, EDGE_FRACT_BITS);
                let w1_min: Fixed<EDGE_FRACT_BITS> = Fixed::from_raw((w1_min_raw >> EDGE_FRACT_BITS) as _, EDGE_FRACT_BITS);
                let w2_min: Fixed<EDGE_FRACT_BITS> = Fixed::from_raw((w2_min_raw >> EDGE_FRACT_BITS) as _, EDGE_FRACT_BITS);

                // TODO: Don't divide, reciprocal multiply
                let w_min: Iv3<DEFAULT_FRACT_BITS> = Iv3::new(w0_min, w1_min, w2_min).div_mixed(scaled_area);

                let r_min: Fixed<COLOR_COMP_BITS> = r.dot_mixed(w_min);
                let g_min: Fixed<COLOR_COMP_BITS> = g.dot_mixed(w_min);
                let b_min: Fixed<COLOR_COMP_BITS> = b.dot_mixed(w_min);
                let a_min: Fixed<COLOR_COMP_BITS> = a.dot_mixed(w_min);
                triangle.r_min = r_min.into_raw(COLOR_COMP_BITS) as _;
                triangle.g_min = g_min.into_raw(COLOR_COMP_BITS) as _;
                triangle.b_min = b_min.into_raw(COLOR_COMP_BITS) as _;
                triangle.a_min = a_min.into_raw(COLOR_COMP_BITS) as _;

                let w_inverse_min: Fixed<W_INVERSE_FRACT_BITS> = w_inverse.dot_mixed(w_min);
                triangle.w_inverse_min = w_inverse_min.into_raw(W_INVERSE_FRACT_BITS) as _;

                let z_min: Fixed<Z_FRACT_BITS> = z.dot_mixed(w_min);
                triangle.z_min = z_min.into_raw(Z_FRACT_BITS) as _;

                let s_min: Fixed<ST_FRACT_BITS> = s.dot_mixed(w_min);
                let t_min: Fixed<ST_FRACT_BITS> = t.dot_mixed(w_min);
                triangle.s_min = s_min.into_raw(ST_FRACT_BITS) as _;
                triangle.t_min = t_min.into_raw(ST_FRACT_BITS) as _;

                let tile_index = tile_index_y * (WIDTH / TILE_DIM) + tile_index_x;
                self.assembled_triangles[tile_index as usize].push(triangle.clone());
            }
        }

        *total_binning_cycles += env.cycles().wrapping_sub(start_cycles);
    }

    pub fn extract_back_buffer(&mut self) -> Vec<u32> {
        let mut ret = Vec::with_capacity(PIXELS as _);

        for y in 0..HEIGHT {
            for x in 0..WIDTH / 4 {
                let word = self.device.mem_read_word(self.back_buffer_base_addr + ((y * WIDTH) + x * 4) * 4);
                for i in 0..4 {
                    ret.push((word >> (i * 32)) as _);
                }
            }
        }

        ret
    }
}
