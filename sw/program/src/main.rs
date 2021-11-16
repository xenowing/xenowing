#![feature(default_alloc_error_handler)]
#![no_main]
#![no_std]

extern crate alloc;

mod native_device;

use native_device::*;

use color_thrust_interface::device::*;
use color_thrust_interface::params_and_regs::*;

use linalg::*;

use strugl::*;

use xw::{marv, stdio, uart};

use alloc::vec::Vec;

use core::fmt::Write;

#[no_mangle]
fn main() -> ! {
    let mut c = Context::new(NativeDevice::new());

    writeln!(stdio::stdout(), "ready for commands").unwrap();

    loop {
        // TODO: Proper command
        uart::write_u8(0x02);

        loop {
            // TODO: This obviously won't work once NativeDevice is a proper singleton, but it's fine for now!
            let mut device = NativeDevice::new();

            // TODO: Proper command
            match uart::read_u8() {
                0x00 => {
                    // Write word
                    let addr = uart::read_u32_le();
                    let data = uart::read_u32_le();
                    device.write_reg(addr, data);
                }
                0x01 => {
                    // Read word
                    let addr = uart::read_u32_le();
                    let data = device.read_reg(addr);
                    uart::write_u32_le(data);
                }
                0x02 => {
                    // Write tile
                    for i in 0..TILE_DIM * TILE_DIM / 4 {
                        let data = uart::read_u128_le();
                        device.write_color_buffer_word(i, data);
                    }
                }
                0x03 => {
                    // Read tile
                    for i in 0..TILE_DIM * TILE_DIM / 4 {
                        let data = device.read_color_buffer_word(i);
                        uart::write_u128_le(data);
                    }
                }
                0x04 => {
                    // Rasterize
                    let start_cycles = marv::cycles();

                    device.write_reg(REG_START_ADDR, 1); // TODO: Proper value
                    while device.read_reg(REG_STATUS_ADDR) != 0 { // TODO: Proper value
                        // Do nothing
                    }

                    let end_cycles = marv::cycles();
                    let elapsed_cycles = end_cycles - start_cycles;
                    uart::write_u64_le(elapsed_cycles);
                }
                0x05 => {
                    // End frame
                    break;
                }
                0x06 => {
                    // Render and transmit entire frame via strugl
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

                    let start_cycles = marv::cycles();

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

                    let mut model = Im4::identity();
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

                    c.render(&mut v);

                    let end_cycles = marv::cycles();
                    let elapsed_cycles = end_cycles - start_cycles;

                    // TODO: Proper command
                    uart::write_u8(0x03);
                    uart::write_u64_le(elapsed_cycles);

                    for y in 0..HEIGHT {
                        for x in 0..WIDTH {
                            uart::write_u32_le(c.back_buffer[y * WIDTH + x]);
                        }
                    }
                    break;
                }
                command => {
                    panic!("unrecognized command: 0x{:02x}", command);
                }
            }
        }
    }
}
