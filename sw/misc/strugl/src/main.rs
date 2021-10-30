mod device;
mod model_device;
mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}
mod sim_device;
mod strugl;

use device::*;
use strugl::*;

use linalg::*;
use image::GenericImageView;
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

use std::env;
use std::time::Instant;

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

    let mut c = Context::new(&mut *device);

    let tex = image::open("myface.png").unwrap();
    let texture_dim = match tex.width() {
        16 => TextureDim::X16,
        32 => TextureDim::X32,
        64 => TextureDim::X64,
        128 => TextureDim::X128,
        _ => panic!("Unsupported texture size")
    };
    if tex.width() != tex.height() {
        panic!("Non-square textures not supported");
    }
    let mut data = vec![0; (tex.width() * tex.height()) as usize];
    for y in 0..tex.height() {
        for x in 0..tex.width() {
            let texel = tex.get_pixel(x, y);
            let r = texel[0];
            let g = texel[1];
            let b = texel[2];
            let a = texel[3];
            let argb = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | ((b as u32) << 0);
            data[(y * tex.width() + x) as usize] = argb;
        }
    }
    let texture_data = c.alloc_texture_data(texture_dim, &data);
    let texture = c.alloc_texture(texture_data, TextureFilter::Bilinear);

    let start_time = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        fn cube(v: &mut Vec<Vertex>) {
            // Front face
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 0.0),
            });

            // Back face
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 0.0),
            });

            // Left face
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 0.0),
            });

            // Right face
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 0.0),
            });

            // Top face
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 0.0),
            });

            // Bottom face
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, 1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, -1.0, 1.0),
                color: V4::splat(1.0),
                tex_coord: V2::new(0.0, 0.0),
            });
        }

        let frame_time = start_time.elapsed().as_secs_f64();

        c.clear();

        c.depth_test_enable = true;
        c.depth_write_mask_enable = true;

        c.texture = Some(texture.clone());

        c.projection = Im4::perspective(90.0, WIDTH as f32 / HEIGHT as f32, 1.0, 1000.0);

        let mut view = Im4::translation(/*-1.0*/0.0, 0.0, -3.0/*-4.0*/);
        let t = (frame_time * 0.1) as f32;
        view = view * Im4::rotation_x(t * 1.1);
        view = view * Im4::rotation_y(t * 0.47);
        view = view * Im4::rotation_z(t * 0.73);

        /*let mut v = Vec::new();

        let mut model = Im4::identity();
        model = model * Im4::translation(-0.5, 0.0, 0.0);
        let t = (frame_time * 0.1) as f32;
        model = model * Im4::rotation_x(t);
        model = model * Im4::rotation_y(t * 0.67);
        model = model * Im4::rotation_z(t * 0.133);
        c.model_view = view * model;

        c.texture_filter = TextureFilter::Nearest;

        cube(&mut v);

        c.render(&mut v);*/

        let mut rng: Pcg32 = SeedableRng::seed_from_u64(0xfadebabedeadbeef);

        for _ in 0..1/*50*/ {
            let mut v = Vec::new();

            let mut model = Im4::identity();
            //model = model * Im4::translation(0.5, 0.0, 0.0);
            /*let t = (frame_time * 0.2) as f32 + rng.gen::<f32>() * 30.0;
            model = model * Im4::rotation_x(t * 1.1);
            model = model * Im4::rotation_y(t * 0.47);
            model = model * Im4::rotation_z(t * 0.73);
            model = model * Im4::translation(0.0, -0.6 + rng.gen::<f32>() * 1.2, -0.6 + rng.gen::<f32>() * 1.2);
            model = model * Im4::scale(1.0 + rng.gen::<f32>() * 0.5, 0.1 + rng.gen::<f32>() * 0.2, 0.04);
            model = model * Im4::translation(0.5 + rng.gen::<f32>() * 0.5, 0.0, 0.0);*/
            c.model_view = view * model;

            let transparent = false;//rng.gen::<bool>();
            if transparent {
                c.depth_write_mask_enable = false;
                c.blend_src_factor = BlendSrcFactor::One;
                c.blend_dst_factor = BlendDstFactor::One;
            } else {
                c.depth_write_mask_enable = true;
                c.blend_src_factor = BlendSrcFactor::One;
                c.blend_dst_factor = BlendDstFactor::Zero;
            }

            cube(&mut v);

            c.render(&mut v);
        }

        let estimated_frame_cycles = c.estimated_frame_bin_cycles + c.estimated_frame_reg_cycles + c.estimated_frame_xfer_cycles + c.estimated_frame_rasterization_cycles;
        let frame_budget_cycles = 100000000 / 60;
        println!("Est. frame cycles: {} / {} ({:.*}%)", estimated_frame_cycles, frame_budget_cycles, 2, estimated_frame_cycles as f64 / frame_budget_cycles as f64 * 100.0);
        println!("  bin r/w:         {} ({:.*}%)", c.estimated_frame_bin_cycles, 2, c.estimated_frame_bin_cycles as f64 / estimated_frame_cycles as f64 * 100.0);
        println!("  regs:            {} ({:.*}%)", c.estimated_frame_reg_cycles, 2, c.estimated_frame_reg_cycles as f64 / estimated_frame_cycles as f64 * 100.0);
        println!("  xfer:            {} ({:.*}%)", c.estimated_frame_xfer_cycles, 2, c.estimated_frame_xfer_cycles as f64 / estimated_frame_cycles as f64 * 100.0);
        println!("  rasterization:   {} ({:.*}%)", c.estimated_frame_rasterization_cycles, 2, c.estimated_frame_rasterization_cycles as f64 / estimated_frame_cycles as f64 * 100.0);

        let mut flipped_buffer: Vec<u32> = Vec::with_capacity(c.back_buffer.len());
        for y in (0..HEIGHT).rev() {
            flipped_buffer.extend_from_slice(&c.back_buffer[y * WIDTH..(y + 1) * WIDTH]);
        }
        window.update_with_buffer(&flipped_buffer, WIDTH, HEIGHT).unwrap();
    }
}
