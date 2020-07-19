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

struct Context<'a> {
    device: &'a mut dyn Device,

    back_buffer: Vec<u32>,
    depth_buffer: Vec<u16>,

    depth_test_enable: bool,
    depth_write_mask_enable: bool,

    model_view: Matrix,
    projection: Matrix,

    verts: Vec<Vertex>,
}

impl<'a> Context<'a> {
    fn new(device: &'a mut dyn Device) -> Context<'a> {
        Context {
            device,

            back_buffer: vec![0; PIXELS],
            depth_buffer: vec![0xffff; PIXELS],

            depth_test_enable: false,
            depth_write_mask_enable: false,

            model_view: Matrix::identity(),
            projection: Matrix::identity(),

            verts: Vec::new(),
        }
    }

    fn render(&mut self) {
        // Transformation
        for vert in self.verts.iter_mut() {
            let object = vert.position;
            let eye = self.model_view * object;
            let clip = self.projection * eye;
            vert.position = clip;
        }

        // Primitive assembly
        for i in (0..self.verts.len()).step_by(3) {
            self.assemble_triangle([self.verts[i + 0], self.verts[i + 1], self.verts[i + 2]])
        }

        // Lol
        self.verts.clear();
    }

    fn assemble_triangle(&mut self, mut verts: [Vertex; 3]) {
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

        self.device.write_reg(
            REG_DEPTH_SETTINGS_ADDR,
            (if self.depth_test_enable { 1 } else { 0 } << REG_DEPTH_TEST_ENABLE_BIT) |
            (if self.depth_write_mask_enable { 1 } else { 0 } << REG_DEPTH_WRITE_MASK_ENABLE_BIT));

        for vert in verts.iter_mut() {
            vert.color = vert.color * 255.0;
        }

        let texture_dims = Vec2::splat(16.0); // TODO: Proper value and default if no texture is enabled
        let st_bias = -0.5; // Offset to sample texel centers // TODO: This depends on filtering chosen; 0 for nearest, -0.5 for bilinear
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

        let w_fract_bits = 8;
        self.device.write_reg(REG_W0_DX_ADDR, to_fixed(w0_dx, w_fract_bits) as _);
        self.device.write_reg(REG_W1_DX_ADDR, to_fixed(w1_dx, w_fract_bits) as _);
        self.device.write_reg(REG_W2_DX_ADDR, to_fixed(w2_dx, w_fract_bits) as _);
        self.device.write_reg(REG_W0_DY_ADDR, to_fixed(w0_dy, w_fract_bits) as _);
        self.device.write_reg(REG_W1_DY_ADDR, to_fixed(w1_dy, w_fract_bits) as _);
        self.device.write_reg(REG_W2_DY_ADDR, to_fixed(w2_dy, w_fract_bits) as _);

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
        let color_fract_bits = 12;
        self.device.write_reg(REG_R_DX_ADDR, to_fixed(r_dx, color_fract_bits) as _);
        self.device.write_reg(REG_G_DX_ADDR, to_fixed(g_dx, color_fract_bits) as _);
        self.device.write_reg(REG_B_DX_ADDR, to_fixed(b_dx, color_fract_bits) as _);
        self.device.write_reg(REG_A_DX_ADDR, to_fixed(a_dx, color_fract_bits) as _);
        self.device.write_reg(REG_R_DY_ADDR, to_fixed(r_dy, color_fract_bits) as _);
        self.device.write_reg(REG_G_DY_ADDR, to_fixed(g_dy, color_fract_bits) as _);
        self.device.write_reg(REG_B_DY_ADDR, to_fixed(b_dy, color_fract_bits) as _);
        self.device.write_reg(REG_A_DY_ADDR, to_fixed(a_dy, color_fract_bits) as _);

        let w_inverse_dx = 1.0 / verts[0].position.w() * w0_dx + 1.0 / verts[1].position.w() * w1_dx + 1.0 / verts[2].position.w() * w2_dx;
        let w_inverse_dy = 1.0 / verts[0].position.w() * w0_dy + 1.0 / verts[1].position.w() * w1_dy + 1.0 / verts[2].position.w() * w2_dy;
        self.device.write_reg(REG_W_INVERSE_DX_ADDR, to_fixed(w_inverse_dx, W_INVERSE_FRACT_BITS) as _);
        self.device.write_reg(REG_W_INVERSE_DY_ADDR, to_fixed(w_inverse_dy, W_INVERSE_FRACT_BITS) as _);

        let z_dx = window_verts[0].z() * w0_dx + window_verts[1].z() * w1_dx + window_verts[2].z() * w2_dx;
        let z_dy = window_verts[0].z() * w0_dy + window_verts[1].z() * w1_dy + window_verts[2].z() * w2_dy;
        self.device.write_reg(REG_Z_DX_ADDR, to_fixed(z_dx, Z_FRACT_BITS) as _);
        self.device.write_reg(REG_Z_DY_ADDR, to_fixed(z_dy, Z_FRACT_BITS) as _);

        let s_dx = verts[0].tex_coord.x() * w0_dx + verts[1].tex_coord.x() * w1_dx + verts[2].tex_coord.x() * w2_dx;
        let t_dx = verts[0].tex_coord.y() * w0_dx + verts[1].tex_coord.y() * w1_dx + verts[2].tex_coord.y() * w2_dx;
        let s_dy = verts[0].tex_coord.x() * w0_dy + verts[1].tex_coord.x() * w1_dy + verts[2].tex_coord.x() * w2_dy;
        let t_dy = verts[0].tex_coord.y() * w0_dy + verts[1].tex_coord.y() * w1_dy + verts[2].tex_coord.y() * w2_dy;
        self.device.write_reg(REG_S_DX_ADDR, to_fixed(s_dx, ST_FRACT_BITS) as _);
        self.device.write_reg(REG_T_DX_ADDR, to_fixed(t_dx, ST_FRACT_BITS) as _);
        self.device.write_reg(REG_S_DY_ADDR, to_fixed(s_dy, ST_FRACT_BITS) as _);
        self.device.write_reg(REG_T_DY_ADDR, to_fixed(t_dy, ST_FRACT_BITS) as _);

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

                // Copy tile into rasterizer memory
                for y in 0..TILE_DIM as usize {
                    for x in 0..TILE_DIM as usize {
                        let buffer_index = (HEIGHT - 1 - (tile_min_y as usize + y)) * WIDTH + tile_min_x as usize + x;
                        self.device.write_color_buffer_word(y as u32 * TILE_DIM + x as u32, self.back_buffer[buffer_index]);
                        self.device.write_depth_buffer_word(y as u32 * TILE_DIM + x as u32, self.depth_buffer[buffer_index]);
                    }
                }

                let p = Vec2::new(tile_min_x as f32, tile_min_y as f32) + 0.5; // Offset to sample pixel centers

                // TODO: Proper top/left fill rule
                let w0_min = orient2d(Vec2::new(window_verts[1].x(), window_verts[1].y()), Vec2::new(window_verts[2].x(), window_verts[2].y()), p);
                let w1_min = orient2d(Vec2::new(window_verts[2].x(), window_verts[2].y()), Vec2::new(window_verts[0].x(), window_verts[0].y()), p);
                let w2_min = orient2d(Vec2::new(window_verts[0].x(), window_verts[0].y()), Vec2::new(window_verts[1].x(), window_verts[1].y()), p);
                self.device.write_reg(REG_W0_MIN_ADDR, to_fixed(w0_min, w_fract_bits) as _);
                self.device.write_reg(REG_W1_MIN_ADDR, to_fixed(w1_min, w_fract_bits) as _);
                self.device.write_reg(REG_W2_MIN_ADDR, to_fixed(w2_min, w_fract_bits) as _);

                let w0_min = w0_min / scaled_area;
                let w1_min = w1_min / scaled_area;
                let w2_min = w2_min / scaled_area;

                let r_min = verts[0].color.x() * w0_min + verts[1].color.x() * w1_min + verts[2].color.x() * w2_min;
                let g_min = verts[0].color.y() * w0_min + verts[1].color.y() * w1_min + verts[2].color.y() * w2_min;
                let b_min = verts[0].color.z() * w0_min + verts[1].color.z() * w1_min + verts[2].color.z() * w2_min;
                let a_min = verts[0].color.w() * w0_min + verts[1].color.w() * w1_min + verts[2].color.w() * w2_min;
                self.device.write_reg(REG_R_MIN_ADDR, to_fixed(r_min, color_fract_bits) as _);
                self.device.write_reg(REG_G_MIN_ADDR, to_fixed(g_min, color_fract_bits) as _);
                self.device.write_reg(REG_B_MIN_ADDR, to_fixed(b_min, color_fract_bits) as _);
                self.device.write_reg(REG_A_MIN_ADDR, to_fixed(a_min, color_fract_bits) as _);

                let w_inverse_min = 1.0 / verts[0].position.w() * w0_min + 1.0 / verts[1].position.w() * w1_min + 1.0 / verts[2].position.w() * w2_min;
                self.device.write_reg(REG_W_INVERSE_MIN_ADDR, to_fixed(w_inverse_min, W_INVERSE_FRACT_BITS) as _);

                let z_min = window_verts[0].z() * w0_min + window_verts[1].z() * w1_min + window_verts[2].z() * w2_min;
                self.device.write_reg(REG_Z_MIN_ADDR, to_fixed(z_min, Z_FRACT_BITS) as _);

                let s_min = verts[0].tex_coord.x() * w0_min + verts[1].tex_coord.x() * w1_min + verts[2].tex_coord.x() * w2_min;
                let t_min = verts[0].tex_coord.y() * w0_min + verts[1].tex_coord.y() * w1_min + verts[2].tex_coord.y() * w2_min;
                self.device.write_reg(REG_S_MIN_ADDR, to_fixed(s_min, ST_FRACT_BITS) as _);
                self.device.write_reg(REG_T_MIN_ADDR, to_fixed(t_min, ST_FRACT_BITS) as _);

                // Rasterize
                self.device.write_reg(REG_START_ADDR, 1);
                while self.device.read_reg(REG_STATUS_ADDR) != 0 {
                    // Do nothing
                }

                // Copy rasterizer memory back to tile
                for y in 0..TILE_DIM as usize {
                    for x in 0..TILE_DIM as usize {
                        let buffer_index = (HEIGHT - 1 - (tile_min_y as usize + y)) * WIDTH + tile_min_x as usize + x;
                        self.back_buffer[buffer_index] = self.device.read_color_buffer_word(y as u32 * TILE_DIM + x as u32);
                        self.depth_buffer[buffer_index] = self.device.read_depth_buffer_word(y as u32 * TILE_DIM + x as u32);
                    }
                }
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
        let frame_time = start_time.elapsed().as_secs_f64();

        // Upload texture
        //  Interleave texels for different tex memories to allow single-cycle filtered texel reads
        for block_y in 0..16 / 2 {
            for block_x in 0..16 / 2 {
                for y in 0..2 {
                    for x in 0..2 {
                        let texel_x = block_x * 2 + x;
                        let texel_y = block_y * 2 + y;
                        let block_offset = block_y * (16 / 2) + block_x;
                        let block_index = y * 2 + x;
                        let addr = block_offset * 4 + block_index;
                        let texel = tex.get_pixel(texel_x, 15 - texel_y);
                        let r = texel[0];
                        let g = texel[1];
                        let b = texel[2];
                        let a = texel[3];
                        let argb = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | ((b as u32) << 0);
                        device.write_tex_buffer_word(addr, argb);
                    }
                }
            }
        }

        fn cube(c: &mut Context) {
            // Front face
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });

            // Back face
            c.verts.push(Vertex {
                position: Vec4::new(1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });

            // Left face
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });

            // Right face
            c.verts.push(Vertex {
                position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });

            // Top face
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, 1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, 1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });

            // Bottom face
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, -1.0, 1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            });
            c.verts.push(Vertex {
                position: Vec4::new(-1.0, -1.0, -1.0, 1.0),
                color: Vec4::splat(1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            });
        }

        let mut c = Context::new(&mut *device);

        c.depth_test_enable = true;
        c.depth_write_mask_enable = true;

        c.projection = Matrix::perspective(90.0, WIDTH as f32 / HEIGHT as f32, 1.0, 1000.0);

        let view = Matrix::translation(0.0, 0.0, -3.0);

        let mut model = Matrix::identity();
        model = model * Matrix::translation(-0.5, 0.0, 0.0);
        let t = (frame_time * 0.1) as f32;
        model = model * Matrix::rotation_x(t);
        model = model * Matrix::rotation_y(t * 0.67);
        model = model * Matrix::rotation_z(t * 0.133);
        c.model_view = view * model;

        cube(&mut c);

        c.render();

        let mut model = Matrix::identity();
        model = model * Matrix::translation(0.5, 0.0, 0.0);
        let t = (frame_time * 0.1) as f32;
        model = model * Matrix::rotation_x(t * 1.1);
        model = model * Matrix::rotation_y(t * 0.47);
        model = model * Matrix::rotation_z(t * 0.73);
        c.model_view = view * model;

        cube(&mut c);

        c.render();

        window.update_with_buffer(&c.back_buffer, WIDTH, HEIGHT).unwrap();
    }
}
