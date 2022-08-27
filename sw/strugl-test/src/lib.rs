#![no_std]

extern crate alloc;

use color_thrust_interface::device::*;

use env::*;

use linalg::*;

use strugl::*;

use core::fmt::Write;

use alloc::vec::Vec;

pub struct StruglTest;

impl StruglTest {
    pub fn new<D: Device, W: Write, E: Environment<W>>(_c: &mut Context<D>, _env: &E) -> StruglTest {
        StruglTest
    }

    pub fn render_frame<D: Device, W: Write, E: Environment<W>>(&mut self, c: &mut Context<D>, env: &E) {
        fn cube(v: &mut Vec<Vertex>) {
            let red = Iv4::new(1.0, 0.0, 0.0, 1.0);
            let green = Iv4::new(0.0, 1.0, 0.0, 1.0);
            let blue = Iv4::new(0.0, 0.0, 1.0, 1.0);
            let white = Iv4::splat(1.0);

            // Front face
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, 1.0, 1.0),
                color: red,
                tex_coord: Iv2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, 1.0, 1.0),
                color: green,
                tex_coord: Iv2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, 1.0, 1.0),
                color: white,
                tex_coord: Iv2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, 1.0, 1.0),
                color: white,
                tex_coord: Iv2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, 1.0, 1.0),
                color: blue,
                tex_coord: Iv2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, 1.0, 1.0),
                color: red,
                tex_coord: Iv2::new(0.0, 0.0),
            });

            // Back face
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, -1.0, 1.0),
                color: red,
                tex_coord: Iv2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, -1.0, 1.0),
                color: green,
                tex_coord: Iv2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, -1.0, 1.0),
                color: white,
                tex_coord: Iv2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, -1.0, 1.0),
                color: white,
                tex_coord: Iv2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, -1.0, 1.0),
                color: blue,
                tex_coord: Iv2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, -1.0, 1.0),
                color: red,
                tex_coord: Iv2::new(0.0, 0.0),
            });

            // Left face
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, -1.0, 1.0),
                color: red,
                tex_coord: Iv2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, 1.0, 1.0),
                color: green,
                tex_coord: Iv2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, 1.0, 1.0),
                color: white,
                tex_coord: Iv2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, 1.0, 1.0),
                color: white,
                tex_coord: Iv2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, -1.0, 1.0),
                color: blue,
                tex_coord: Iv2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, -1.0, 1.0),
                color: red,
                tex_coord: Iv2::new(0.0, 0.0),
            });

            // Right face
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, 1.0, 1.0),
                color: red,
                tex_coord: Iv2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, -1.0, 1.0),
                color: green,
                tex_coord: Iv2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, -1.0, 1.0),
                color: white,
                tex_coord: Iv2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, -1.0, 1.0),
                color: white,
                tex_coord: Iv2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, 1.0, 1.0),
                color: blue,
                tex_coord: Iv2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, 1.0, 1.0),
                color: red,
                tex_coord: Iv2::new(0.0, 0.0),
            });

            // Top face
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, 1.0, 1.0),
                color: red,
                tex_coord: Iv2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, 1.0, 1.0),
                color: green,
                tex_coord: Iv2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, -1.0, 1.0),
                color: white,
                tex_coord: Iv2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, 1.0, -1.0, 1.0),
                color: white,
                tex_coord: Iv2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, -1.0, 1.0),
                color: blue,
                tex_coord: Iv2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, 1.0, 1.0, 1.0),
                color: red,
                tex_coord: Iv2::new(0.0, 0.0),
            });

            // Bottom face
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, -1.0, 1.0),
                color: red,
                tex_coord: Iv2::new(0.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, -1.0, 1.0),
                color: green,
                tex_coord: Iv2::new(1.0, 0.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, 1.0, 1.0),
                color: white,
                tex_coord: Iv2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(1.0, -1.0, 1.0, 1.0),
                color: white,
                tex_coord: Iv2::new(1.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, 1.0, 1.0),
                color: blue,
                tex_coord: Iv2::new(0.0, 1.0),
            });
            v.push(Vertex {
                position: Iv4::new(-1.0, -1.0, -1.0, 1.0),
                color: red,
                tex_coord: Iv2::new(0.0, 0.0),
            });
        }

        let mut v = Vec::new();
        cube(&mut v);

        let frame_time = 6.0;//start_time.elapsed().as_secs_f64();

        c.clear();

        c.depth_test_enable = true;
        c.depth_write_mask_enable = true;

        //c.texture = Some(texture.clone());

        c.projection = Im4::perspective(90.0, WIDTH as f32 / HEIGHT as f32, 1.0, 1000.0);

        let mut view = Im4::translation(/*-1.0*/0.0, 0.0, -3.0/*-4.0*/);
        let t = (frame_time * 0.1) as f32;
        view *= Im4::rotation_x(t * 1.1);
        view *= Im4::rotation_y(t * 0.47);
        view *= Im4::rotation_z(t * 0.73);

        /*let mut v = Vec::new();

        let mut model = Im4::identity();
        model *= Im4::translation(-0.5, 0.0, 0.0);
        let t = (frame_time * 0.1) as f32;
        model *= Im4::rotation_x(t);
        model *= Im4::rotation_y(t * 0.67);
        model *= Im4::rotation_z(t * 0.133);
        c.model_view = view * model;

        c.texture_filter = TextureFilter::Nearest;

        cube(&mut v);

        c.render(&mut v);*/

        //let mut rng: Pcg32 = SeedableRng::seed_from_u64(0xfadebabedeadbeef);

        let /*mut */model = Im4::identity();
        //model *= Im4::translation(0.5, 0.0, 0.0);
        /*let t = (frame_time * 0.2) as f32 + rng.gen::<f32>() * 30.0;
        model *= Im4::rotation_x(t * 1.1);
        model *= Im4::rotation_y(t * 0.47);
        model *= Im4::rotation_z(t * 0.73);
        model *= Im4::translation(0.0, -0.6 + rng.gen::<f32>() * 1.2, -0.6 + rng.gen::<f32>() * 1.2);
        model *= Im4::scale(1.0 + rng.gen::<f32>() * 0.5, 0.1 + rng.gen::<f32>() * 0.2, 0.04);
        model *= Im4::translation(0.5 + rng.gen::<f32>() * 0.5, 0.0, 0.0);*/
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

        let mut total_primitive_assembly_cycles = 0;
        let mut total_binning_cycles = 0;
        let stats = c.render(&v, &mut total_primitive_assembly_cycles, &mut total_binning_cycles, env);

        writeln!(env.stdout(), "Vertex transformation cycles: {}", stats.vertex_transformation_cycles).unwrap();
        writeln!(env.stdout(), "Primitive assembly and binning cycles: {}", stats.primitive_assembly_and_binning_cycles).unwrap();
        writeln!(env.stdout(), " - Primitive assembly cycles: {}", total_primitive_assembly_cycles).unwrap();
        writeln!(env.stdout(), " - Binning cycles: {}", total_binning_cycles).unwrap();
        writeln!(env.stdout(), "Num nonempty tiles: {}", stats.num_nonempty_tiles).unwrap();
        writeln!(env.stdout(), "Total tile xfer cycles: {}", stats.total_tile_xfer_cycles).unwrap();
        writeln!(env.stdout(), "Total rasterization cycles: {}", stats.total_rasterization_cycles).unwrap();
    }
}
