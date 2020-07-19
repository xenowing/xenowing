#![feature(stdarch)]

mod device;
mod matrix;
mod model_device;
mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}
mod sim_device;
mod vec2;
mod vec3;
mod vec4;

use device::*;
use matrix::*;
use vec2::*;
use vec3::*;
use vec4::*;

use image::GenericImageView;
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use rtl::color_thrust::*;

use std::env;
use std::time::Instant;

const WIDTH: usize = 16 * 8;//320;
const HEIGHT: usize = 16 * 8;//240;
const PIXELS: usize = WIDTH * HEIGHT;

#[derive(Clone, Copy)]
struct Vertex {
    position: Vec4,
    color: Vec4,
    tex_coord: Vec2,
}

enum TextureFilter {
    Nearest,
    Bilinear,
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

struct Context<'a> {
    device: &'a mut dyn Device,

    back_buffer: Vec<u32>,
    depth_buffer: Vec<u16>,

    depth_test_enable: bool,
    depth_write_mask_enable: bool,

    // TODO: Move to texture object
    texture_filter: TextureFilter,

    model_view: Matrix,
    projection: Matrix,

    assembled_triangles: Vec<Vec<Triangle>>,

    estimated_frame_reg_cycles: u64,
    estimated_frame_xfer_cycles: u64,
    estimated_frame_rasterization_cycles: u64,
}

impl<'a> Context<'a> {
    fn new(device: &'a mut dyn Device) -> Context<'a> {
        Context {
            device,

            back_buffer: vec![0; PIXELS],
            depth_buffer: vec![0xffff; PIXELS],

            depth_test_enable: false,
            depth_write_mask_enable: false,

            texture_filter: TextureFilter::Nearest,

            model_view: Matrix::identity(),
            projection: Matrix::identity(),

            // TODO: Fixed capacity and splitting drawcalls on overflow
            assembled_triangles: vec![Vec::new(); PIXELS / TILE_PIXELS as usize],

            estimated_frame_reg_cycles: 0,
            estimated_frame_xfer_cycles: 0,
            estimated_frame_rasterization_cycles: 0,
        }
    }

    fn render(&mut self, verts: &mut Vec<Vertex>) {
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

        self.device.write_reg(
            REG_TEXTURE_SETTINGS_ADDR,
            match self.texture_filter {
                TextureFilter::Nearest => REG_TEXTURE_SETTINGS_FILTER_SELECT_NEAREST,
                TextureFilter::Bilinear => REG_TEXTURE_SETTINGS_FILTER_SELECT_BILINEAR,
            } << REG_TEXTURE_SETTINGS_FILTER_SELECT_BIT);
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

                assembled_triangles.clear();

                // Copy rasterizer memory back to tile
                for y in 0..TILE_DIM as usize {
                    for x in 0..TILE_DIM as usize / 4 {
                        let buffer_index = (HEIGHT - 1 - (tile_min_y as usize + y)) * WIDTH + tile_min_x as usize + x * 4;
                        let word = self.device.read_color_buffer_word(y as u32 * TILE_DIM / 4 + x as u32);
                        for i in 0..4 {
                            self.back_buffer[buffer_index + i] = (word >> (32 * i)) as _;
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
        let mut window_verts = [Vec3::zero(); 3];
        for i in 0..3 {
            let clip = verts[i].position;
            let ndc = Vec3::new(clip.x(), clip.y(), clip.z()) / clip.w();
            let viewport_near = 0.0;
            let viewport_far = 1.0;
            let viewport_scale = Vec3::new(viewport_width as f32 / 2.0, viewport_height as f32 / 2.0, (viewport_far - viewport_near) / 2.0);
            let viewport_bias = Vec3::new(viewport_x as f32 + viewport_width as f32 / 2.0, viewport_y as f32 + viewport_height as f32 / 2.0, (viewport_far + viewport_near) / 2.0);
            window_verts[i] = ndc * viewport_scale + viewport_bias;
        }

        fn orient2d(a: Vec2, b: Vec2, c: Vec2) -> f32 {
            (b.x() - a.x()) * (c.y() - a.y()) - (b.y() - a.y()) * (c.x() - a.x())
        }

        let /*mut */scaled_area = orient2d(
            Vec2::new(window_verts[0].x(), window_verts[0].y()),
            Vec2::new(window_verts[1].x(), window_verts[1].y()),
            Vec2::new(window_verts[2].x(), window_verts[2].y()));

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

        let texture_dims = Vec2::splat(16.0); // TODO: Proper value and default if no texture is enabled
        // Offset to sample texel centers
        let st_bias = match self.texture_filter {
            TextureFilter::Nearest => 0.0,
            TextureFilter::Bilinear => -0.5,
        };
        for vert in verts.iter_mut() {
            vert.tex_coord = (vert.tex_coord * texture_dims + st_bias) / vert.position.w();
        }

        let mut bb_min = Vec2::new(window_verts[0].x(), window_verts[0].y());
        let mut bb_max = bb_min;
        for i in 1..verts.len() {
            bb_min = bb_min.min(Vec2::new(window_verts[i].x(), window_verts[i].y()));
            bb_max = bb_max.max(Vec2::new(window_verts[i].x(), window_verts[i].y()));
        }
        bb_min = bb_min.max(Vec2::new(viewport_x as f32, viewport_y as f32));
        bb_max = bb_max.min(Vec2::new((viewport_x + viewport_width as i32 - 1) as f32, (viewport_y + viewport_height as i32 - 1) as f32));
        bb_min = bb_min.max(Vec2::zero());
        bb_max = bb_max.min(Vec2::new((WIDTH - 1) as f32, (HEIGHT - 1) as f32));
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

                let p = Vec2::new(tile_min_x as f32, tile_min_y as f32) + 0.5; // Offset to sample pixel centers

                // TODO: Proper top/left fill rule
                let w0_min = orient2d(Vec2::new(window_verts[1].x(), window_verts[1].y()), Vec2::new(window_verts[2].x(), window_verts[2].y()), p);
                let w1_min = orient2d(Vec2::new(window_verts[2].x(), window_verts[2].y()), Vec2::new(window_verts[0].x(), window_verts[0].y()), p);
                let w2_min = orient2d(Vec2::new(window_verts[0].x(), window_verts[0].y()), Vec2::new(window_verts[1].x(), window_verts[1].y()), p);
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
            }
        }
    }
}

fn main() {
    let device_type = env::args().skip(1).nth(0).expect("No device type argument provided");

    let mut device: Box<dyn Device> = match device_type.as_str() {
        "model" => Box::new(model_device::ModelDevice::new()),
        "sim" => Box::new(sim_device::SimDevice::new()),
        _ => panic!("Invalid device type argument")
    };

    let mut window = Window::new("strugl", WIDTH, HEIGHT, WindowOptions {
        scale: Scale::X4,
        scale_mode: ScaleMode::AspectRatioStretch,
        ..WindowOptions::default()
    }).unwrap();

    let tex = image::open("tex.png").unwrap();

    let start_time = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        fn cube(v: &mut Vec<Vertex>) {
            // Front face
            v.push(Vertex {
                position: Vec4::new(-1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });

            // Back face
            v.push(Vertex {
                position: Vec4::new(1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });

            // Left face
            v.push(Vertex {
                position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });

            // Right face
            v.push(Vertex {
                position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });

            // Top face
            v.push(Vertex {
                position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });

            // Bottom face
            v.push(Vertex {
                position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
        }

        let frame_time = start_time.elapsed().as_secs_f64();

        let mut c = Context::new(&mut *device);

        // Upload texture
        //  Interleave texels for different tex memories to allow single-cycle filtered texel reads
        for block_y in 0..16 / 2 {
            for block_x in 0..16 / 2 {
                let mut word = 0;
                for y in 0..2 {
                    for x in 0..2 {
                        let texel_x = block_x * 2 + x;
                        let texel_y = block_y * 2 + y;
                        let block_index = y * 2 + x;
                        let texel = tex.get_pixel(texel_x, 15 - texel_y);
                        let r = texel[0];
                        let g = texel[1];
                        let b = texel[2];
                        let a = texel[3];
                        let argb = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | ((b as u32) << 0);
                        word |= (argb as u128) << (block_index * 32);
                    }
                }
                let addr = block_y * (16 / 2) + block_x;
                c.device.write_tex_buffer_word(addr, word);

                c.estimated_frame_xfer_cycles += 1;
            }
        }

        c.depth_test_enable = true;
        c.depth_write_mask_enable = true;

        c.projection = Matrix::perspective(90.0, WIDTH as f32 / HEIGHT as f32, 1.0, 1000.0);

        let view = Matrix::translation(0.0, 0.0, -3.0);

        let mut v = Vec::new();

        let mut model = Matrix::identity();
        model = model * Matrix::translation(-0.5, 0.0, 0.0);
        let t = (frame_time * 0.1) as f32;
        model = model * Matrix::rotation_x(t);
        model = model * Matrix::rotation_y(t * 0.67);
        model = model * Matrix::rotation_z(t * 0.133);
        c.model_view = view * model;

        c.texture_filter = TextureFilter::Nearest;

        cube(&mut v);

        c.render(&mut v);

        let mut v = Vec::new();

        let mut model = Matrix::identity();
        model = model * Matrix::translation(0.5, 0.0, 0.0);
        let t = (frame_time * 0.1) as f32;
        model = model * Matrix::rotation_x(t * 1.1);
        model = model * Matrix::rotation_y(t * 0.47);
        model = model * Matrix::rotation_z(t * 0.73);
        c.model_view = view * model;

        c.texture_filter = TextureFilter::Bilinear;

        cube(&mut v);

        c.render(&mut v);

        let estimated_frame_cycles = c.estimated_frame_reg_cycles + c.estimated_frame_xfer_cycles + c.estimated_frame_rasterization_cycles;
        let frame_budget_cycles = 100000000 / 60;
        println!("Est. frame cycles: {} / {} ({:.*}%)", estimated_frame_cycles, frame_budget_cycles, 2, estimated_frame_cycles as f64 / frame_budget_cycles as f64 * 100.0);
        println!("  regs:            {} ({:.*}%)", c.estimated_frame_reg_cycles, 2, c.estimated_frame_reg_cycles as f64 / estimated_frame_cycles as f64 * 100.0);
        println!("  xfer:            {} ({:.*}%)", c.estimated_frame_xfer_cycles, 2, c.estimated_frame_xfer_cycles as f64 / estimated_frame_cycles as f64 * 100.0);
        println!("  rasterization:   {} ({:.*}%)", c.estimated_frame_rasterization_cycles, 2, c.estimated_frame_rasterization_cycles as f64 / estimated_frame_cycles as f64 * 100.0);

        window.update_with_buffer(&c.back_buffer, WIDTH, HEIGHT).unwrap();
    }
}
