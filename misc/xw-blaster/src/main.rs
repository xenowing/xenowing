#![feature(stdarch)]

mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}
mod vec2;
mod vec4;

use modules::*;
use vec2::*;
use vec4::*;

use image::GenericImageView;
use minifb::{Scale, ScaleMode, Window, WindowOptions};
use rtl::color_thrust::*;
use serialport::prelude::*;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use std::env;
use std::fs;
use std::io::{self, Write};
use std::str;
use std::sync::mpsc::{self, channel, Receiver, Sender};
use std::thread;

const WIDTH: usize = 16 * 8;//320;
const HEIGHT: usize = 16 * 8;//240;
const PIXELS: usize = WIDTH * HEIGHT;

#[derive(Clone, Copy)]
struct Vertex {
    position: Vec2,
    color: Vec4,
    tex_coord: Vec2,
}

#[derive(Debug)]
enum Error {
    Io(io::Error),
    Other(String),
    Recv(mpsc::RecvError),
    Send(mpsc::SendError<u8>),
    SerialPort(serialport::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<String> for Error {
    fn from(error: String) -> Error {
        Error::Other(error)
    }
}

impl From<mpsc::RecvError> for Error {
    fn from(error: mpsc::RecvError) -> Error {
        Error::Recv(error)
    }
}

impl From<mpsc::SendError<u8>> for Error {
    fn from(error: mpsc::SendError<u8>) -> Error {
        Error::Send(error)
    }
}

impl From<serialport::Error> for Error {
    fn from(error: serialport::Error) -> Error {
        Error::SerialPort(error)
    }
}

trait Device {
    fn read_byte(&mut self) -> Result<u8, Error>;
    fn write_byte(&mut self, value: u8) -> Result<(), Error>;

    fn read_u32(&mut self) -> Result<u32, Error> {
        let mut ret = 0x00;
        ret |= (self.read_byte()? as u32) << 0;
        ret |= (self.read_byte()? as u32) << 8;
        ret |= (self.read_byte()? as u32) << 16;
        ret |= (self.read_byte()? as u32) << 24;

        Ok(ret)
    }

    fn write_u32(&mut self, value: u32) -> Result<(), Error> {
        self.write_byte((value >> 0) as _)?;
        self.write_byte((value >> 8) as _)?;
        self.write_byte((value >> 16) as _)?;
        self.write_byte((value >> 24) as _)?;

        Ok(())
    }
}

struct SimDevice {
    host_command_rx: Receiver<u8>,
    host_response_tx: Sender<u8>,
}

impl SimDevice {
    fn new() -> SimDevice {
        let (host_command_tx, host_command_rx) = channel();
        let (host_response_tx, host_response_rx) = channel();

        // TODO: This is leaky, but I guess it doesn't matter :)
        thread::spawn(move|| {
            let mut leds = 0b000;

            let mut is_sending_byte = false;

            let mut top = Top::new();

            let mut is_first_cycle = true;
            loop {
                if is_first_cycle {
                    top.reset();

                    is_first_cycle = false;
                } else {
                    top.posedge_clk();

                    let new_leds = top.leds;
                    if new_leds != leds {
                        println!("LEDs updated: 0b{:08b} -> 0b{:08b}", leds, new_leds);
                        leds = new_leds;
                    }

                    if top.uart_tx_data_valid {
                        host_command_tx.send(top.uart_tx_data as _).unwrap();
                    }

                    // TODO: This isn't necessarily the best way to use this interface, but it should work :)
                    if is_sending_byte && top.uart_rx_ready {
                        is_sending_byte = false;
                        top.uart_rx_enable = false;
                    }
                    if !is_sending_byte {
                        if let Ok(value) = host_response_rx.try_recv() {
                            is_sending_byte = true;
                            top.uart_rx_enable = true;
                            top.uart_rx_data = value as u32;
                        }
                    }
                }

                top.prop();
            }
        });

        SimDevice {
            host_command_rx,
            host_response_tx,
        }
    }
}

impl Device for SimDevice {
    fn read_byte(&mut self) -> Result<u8, Error> {
        Ok(self.host_command_rx.recv()?)
    }

    fn write_byte(&mut self, value: u8) -> Result<(), Error> {
        self.host_response_tx.send(value)?;

        Ok(())
    }
}

struct SerialDevice {
    port: Box<dyn SerialPort>,
}

impl SerialDevice {
    fn new(port_name: String) -> Result<SerialDevice, Error> {
        let baud_rate: u32 = 460800;

        let mut settings: SerialPortSettings = Default::default();
        settings.baud_rate = baud_rate.into();

        let port = serialport::open_with_settings(&port_name, &settings)?;
        let actual_baud_rate = port.baud_rate()?;
        if actual_baud_rate != baud_rate {
            return Err(format!("Unable to achieve specified baud rate: got {}, expected {}", actual_baud_rate, baud_rate).into());
        }

        Ok(SerialDevice {
            port,
        })
    }
}

impl Device for SerialDevice {
    fn read_byte(&mut self) -> Result<u8, Error> {
        let mut buf = [0];
        loop {
            match self.port.read(&mut buf) {
                Ok(t) => {
                    if t > 0 {
                        return Ok(buf[0]);
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }

    fn write_byte(&mut self, value: u8) -> Result<(), Error> {
        self.port.write_all(&[value])?;

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let mut device: Box<dyn Device> = if let Some(port_name) = env::args().nth(1) {
        println!("Creating serial device on port {}", port_name);
        Box::new(SerialDevice::new(port_name)?)
    } else {
        println!("Creating sim device");
        Box::new(SimDevice::new())
    };
    println!();

    let mut back_buffer = vec![0xffff00ff; PIXELS];

    let mut window = Window::new("trim", WIDTH, HEIGHT, WindowOptions {
        scale: Scale::X4,
        scale_mode: ScaleMode::AspectRatioStretch,
        ..WindowOptions::default()
    }).unwrap();

    let tex = image::open("tex.png").unwrap();

    println!("XENOWING BLASTER ENGAGED");
    println!("ALL SYSTEMS ARE GO");
    println!();

    loop {
        match device.read_byte()? {
            0x00 => {
                // XW_UART_COMMAND_PUTC
                print!("{}", device.read_byte()? as char);
            }
            0x01 => {
                let mut stdout = StandardStream::stdout(ColorChoice::Always);
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_intense(true))?;

                // File read test
                let mut filename = Vec::new();
                loop {
                    let c = device.read_byte()?;
                    if c == 0 {
                        break;
                    }

                    filename.push(c);
                }
                let filename = str::from_utf8(&filename).unwrap();
                writeln!(&mut stdout, "file requested: {}", filename)?;
                let file = fs::read(filename)?;
                let len = file.len();
                device.write_u32(len as _)?;
                for byte in file {
                    device.write_byte(byte)?;
                }

                stdout.reset()?;
            }
            0x02 => {
                let mut stdout = StandardStream::stdout(ColorChoice::Always);
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_intense(true))?;

                writeln!(&mut stdout, "commands requested, starting frame")?;

                fn write_word(addr: u32, data: u32, device: &mut dyn Device) -> Result<(), Error> {
                    println!("    write_word: addr: {:08x}, data: {:08x}", addr, data);
                    device.write_byte(0x00)?;
                    device.write_u32(addr)?;
                    device.write_u32(data)?;

                    Ok(())
                }

                fn write_reg(addr: u32, data: u32, device: &mut dyn Device) -> Result<(), Error> {
                    write_word(0x04000000 + addr * 16, data, device)
                }

                // Upload texture
                writeln!(&mut stdout, "  write tex mem")?;
                for y in 0..16 {
                    for x in 0..16 {
                        let texel = tex.get_pixel(x, 15 - y);
                        let r = texel[0];
                        let g = texel[1];
                        let b = texel[2];
                        let a = texel[3];
                        let addr = 0x06000000 + (y * 16 + x) * 16;
                        write_word(addr, ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | ((b as u32) << 0), &mut *device)?;
                    }
                }

                for i in 0..1/*5*/ {
                    writeln!(&mut stdout, "  triangle {}", i)?;

                    let mut verts = [
                        /*Vertex {
                            position: Vec2::new(-1.0, -1.0),//Vec2::new(rand::random::<f32>() * 2.0 - 1.0, rand::random::<f32>() * 2.0 - 1.0),
                            color: Vec4::new(1.0, 0.0, 0.0, 1.0) * 255.0,//Vec4::new(rand::random(), rand::random(), rand::random(), rand::random()) * 255.0,
                            tex_coord: Vec2::zero(),
                        },
                        Vertex {
                            position: Vec2::new( 1.0, -1.0),//Vec2::new(rand::random::<f32>() * 2.0 - 1.0, rand::random::<f32>() * 2.0 - 1.0),
                            color: Vec4::new(0.0, 0.0, 1.0, 1.0) * 255.0,//Vec4::new(rand::random(), rand::random(), rand::random(), rand::random()) * 255.0,
                            tex_coord: Vec2::new(1.0, 0.0),
                        },
                        Vertex {
                            position: Vec2::new(1.0, 1.0),//Vec2::new(rand::random::<f32>() * 2.0 - 1.0, rand::random::<f32>() * 2.0 - 1.0),
                            color: Vec4::new(0.0, 1.0, 0.0, 1.0) * 255.0,//Vec4::new(rand::random(), rand::random(), rand::random(), rand::random()) * 255.0,
                            tex_coord: Vec2::new(1.0, 1.0),
                        },*/
                        Vertex {
                            position: Vec2::new(rand::random::<f32>() * 2.0 - 1.0, rand::random::<f32>() * 2.0 - 1.0),
                            color: Vec4::new(rand::random(), rand::random(), rand::random(), rand::random()) * 255.0,
                            tex_coord: Vec2::zero(),
                        },
                        Vertex {
                            position: Vec2::new(rand::random::<f32>() * 2.0 - 1.0, rand::random::<f32>() * 2.0 - 1.0),
                            color: Vec4::new(rand::random(), rand::random(), rand::random(), rand::random()) * 255.0,
                            tex_coord: Vec2::new(1.0, 0.0),
                        },
                        Vertex {
                            position: Vec2::new(rand::random::<f32>() * 2.0 - 1.0, rand::random::<f32>() * 2.0 - 1.0),
                            color: Vec4::new(rand::random(), rand::random(), rand::random(), rand::random()) * 255.0,
                            tex_coord: Vec2::new(1.0, 1.0),
                        },
                    ];

                    let viewport_x = 0;
                    let viewport_y = 0;
                    let viewport_width = WIDTH;
                    let viewport_height = HEIGHT;

                    // Viewport transform
                    let mut window_verts = [Vec2::zero(); 3];
                    for i in 0..3 {
                        let clip = verts[i].position;
                        let ndc = Vec2::new(clip.x(), clip.y());
                        let viewport_scale = Vec2::new(viewport_width as f32 / 2.0, viewport_height as f32 / 2.0);
                        let viewport_bias = Vec2::new(viewport_x as f32 + viewport_width as f32 / 2.0, viewport_y as f32 + viewport_height as f32 / 2.0);
                        window_verts[i] = ndc * viewport_scale + viewport_bias;
                    }

                    fn orient2d(a: Vec2, b: Vec2, c: Vec2) -> f32 {
                        (b.x() - a.x()) * (c.y() - a.y()) - (b.y() - a.y()) * (c.x() - a.x())
                    }

                    let mut scaled_area = orient2d(
                        Vec2::new(window_verts[0].x(), window_verts[0].y()),
                        Vec2::new(window_verts[1].x(), window_verts[1].y()),
                        Vec2::new(window_verts[2].x(), window_verts[2].y()));

                    // Flip backfacing triangles (TODO: Proper back/front face culling)
                    if scaled_area < 0.0 {
                        let temp = verts[0];
                        verts[0] = verts[1];
                        verts[1] = temp;
                        let temp = window_verts[0];
                        window_verts[0] = window_verts[1];
                        window_verts[1] = temp;
                        scaled_area = -scaled_area;
                    }

                    let texture_dims = Vec2::splat(16.0); // TODO: Proper value and default if no texture is enabled
                    let st_bias = -0.5; // Offset to sample texel centers
                    for i in 0..verts.len() {
                        verts[i].tex_coord = (verts[i].tex_coord * texture_dims + st_bias) / 1.0;//verts[i].position.w(); // TODO: Proper w!!!!!!!
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
                    write_reg(REG_W0_DX_ADDR, to_fixed(w0_dx, w_fract_bits) as _, &mut *device)?;
                    write_reg(REG_W1_DX_ADDR, to_fixed(w1_dx, w_fract_bits) as _, &mut *device)?;
                    write_reg(REG_W2_DX_ADDR, to_fixed(w2_dx, w_fract_bits) as _, &mut *device)?;
                    write_reg(REG_W0_DY_ADDR, to_fixed(w0_dy, w_fract_bits) as _, &mut *device)?;
                    write_reg(REG_W1_DY_ADDR, to_fixed(w1_dy, w_fract_bits) as _, &mut *device)?;
                    write_reg(REG_W2_DY_ADDR, to_fixed(w2_dy, w_fract_bits) as _, &mut *device)?;

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
                    write_reg(REG_R_DX_ADDR, to_fixed(r_dx, color_fract_bits) as _, &mut *device)?;
                    write_reg(REG_G_DX_ADDR, to_fixed(g_dx, color_fract_bits) as _, &mut *device)?;
                    write_reg(REG_B_DX_ADDR, to_fixed(b_dx, color_fract_bits) as _, &mut *device)?;
                    write_reg(REG_A_DX_ADDR, to_fixed(a_dx, color_fract_bits) as _, &mut *device)?;
                    write_reg(REG_R_DY_ADDR, to_fixed(r_dy, color_fract_bits) as _, &mut *device)?;
                    write_reg(REG_G_DY_ADDR, to_fixed(g_dy, color_fract_bits) as _, &mut *device)?;
                    write_reg(REG_B_DY_ADDR, to_fixed(b_dy, color_fract_bits) as _, &mut *device)?;
                    write_reg(REG_A_DY_ADDR, to_fixed(a_dy, color_fract_bits) as _, &mut *device)?;

                    // TODO!
                    let w_inverse_dx = 0.0;//1.0 / verts[0].position.w() * w0_dx + 1.0 / verts[1].position.w() * w1_dx + 1.0 / verts[2].position.w() * w2_dx;
                    let w_inverse_dy = 0.0;//1.0 / verts[0].position.w() * w0_dy + 1.0 / verts[1].position.w() * w1_dy + 1.0 / verts[2].position.w() * w2_dy;
                    write_reg(REG_W_INVERSE_DX_ADDR, to_fixed(w_inverse_dx, W_INVERSE_FRACT_BITS) as _, &mut *device)?;
                    write_reg(REG_W_INVERSE_DY_ADDR, to_fixed(w_inverse_dy, W_INVERSE_FRACT_BITS) as _, &mut *device)?;

                    let s_dx = verts[0].tex_coord.x() * w0_dx + verts[1].tex_coord.x() * w1_dx + verts[2].tex_coord.x() * w2_dx;
                    let t_dx = verts[0].tex_coord.y() * w0_dx + verts[1].tex_coord.y() * w1_dx + verts[2].tex_coord.y() * w2_dx;
                    let s_dy = verts[0].tex_coord.x() * w0_dy + verts[1].tex_coord.x() * w1_dy + verts[2].tex_coord.x() * w2_dy;
                    let t_dy = verts[0].tex_coord.y() * w0_dy + verts[1].tex_coord.y() * w1_dy + verts[2].tex_coord.y() * w2_dy;
                    write_reg(REG_S_DX_ADDR, to_fixed(s_dx, ST_FRACT_BITS) as _, &mut *device)?;
                    write_reg(REG_T_DX_ADDR, to_fixed(t_dx, ST_FRACT_BITS) as _, &mut *device)?;
                    write_reg(REG_S_DY_ADDR, to_fixed(s_dy, ST_FRACT_BITS) as _, &mut *device)?;
                    write_reg(REG_T_DY_ADDR, to_fixed(t_dy, ST_FRACT_BITS) as _, &mut *device)?;

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
                            writeln!(&mut stdout, "    write tile")?;
                            device.write_byte(0x02)?;
                            for y in 0..TILE_DIM as usize {
                                for x in 0..TILE_DIM as usize {
                                    let buffer_index = (HEIGHT - 1 - (tile_min_y as usize + y)) * WIDTH + tile_min_x as usize + x;
                                    device.write_u32(back_buffer[buffer_index])?;
                                }
                            }

                            let p = Vec2::new(tile_min_x as f32, tile_min_y as f32) + 0.5; // Offset to sample pixel centers

                            // TODO: Proper top/left fill rule
                            let w0_min = orient2d(Vec2::new(window_verts[1].x(), window_verts[1].y()), Vec2::new(window_verts[2].x(), window_verts[2].y()), p);
                            let w1_min = orient2d(Vec2::new(window_verts[2].x(), window_verts[2].y()), Vec2::new(window_verts[0].x(), window_verts[0].y()), p);
                            let w2_min = orient2d(Vec2::new(window_verts[0].x(), window_verts[0].y()), Vec2::new(window_verts[1].x(), window_verts[1].y()), p);
                            write_reg(REG_W0_MIN_ADDR, to_fixed(w0_min, w_fract_bits) as _, &mut *device)?;
                            write_reg(REG_W1_MIN_ADDR, to_fixed(w1_min, w_fract_bits) as _, &mut *device)?;
                            write_reg(REG_W2_MIN_ADDR, to_fixed(w2_min, w_fract_bits) as _, &mut *device)?;

                            let w0_min = w0_min / scaled_area;
                            let w1_min = w1_min / scaled_area;
                            let w2_min = w2_min / scaled_area;

                            let r_min = verts[0].color.x() * w0_min + verts[1].color.x() * w1_min + verts[2].color.x() * w2_min;
                            let g_min = verts[0].color.y() * w0_min + verts[1].color.y() * w1_min + verts[2].color.y() * w2_min;
                            let b_min = verts[0].color.z() * w0_min + verts[1].color.z() * w1_min + verts[2].color.z() * w2_min;
                            let a_min = verts[0].color.w() * w0_min + verts[1].color.w() * w1_min + verts[2].color.w() * w2_min;
                            write_reg(REG_R_MIN_ADDR, to_fixed(r_min, color_fract_bits) as _, &mut *device)?;
                            write_reg(REG_G_MIN_ADDR, to_fixed(g_min, color_fract_bits) as _, &mut *device)?;
                            write_reg(REG_B_MIN_ADDR, to_fixed(b_min, color_fract_bits) as _, &mut *device)?;
                            write_reg(REG_A_MIN_ADDR, to_fixed(a_min, color_fract_bits) as _, &mut *device)?;

                            // TODO!
                            let w_inverse_min = 1.0 / 1.0;//1.0 / verts[0].position.w() * w0_min + 1.0 / verts[1].position.w() * w1_min + 1.0 / verts[2].position.w() * w2_min;
                            write_reg(REG_W_INVERSE_MIN_ADDR, to_fixed(w_inverse_min, W_INVERSE_FRACT_BITS) as _, &mut *device)?;

                            let s_min = verts[0].tex_coord.x() * w0_min + verts[1].tex_coord.x() * w1_min + verts[2].tex_coord.x() * w2_min;
                            let t_min = verts[0].tex_coord.y() * w0_min + verts[1].tex_coord.y() * w1_min + verts[2].tex_coord.y() * w2_min;
                            write_reg(REG_S_MIN_ADDR, to_fixed(s_min, ST_FRACT_BITS) as _, &mut *device)?;
                            write_reg(REG_T_MIN_ADDR, to_fixed(t_min, ST_FRACT_BITS) as _, &mut *device)?;

                            // Rasterize
                            writeln!(&mut stdout, "    rasterize")?;
                            device.write_byte(0x04)?;
                            let mut elapsed_cycles = 0;
                            elapsed_cycles |= (device.read_byte()? as u64) << 0;
                            elapsed_cycles |= (device.read_byte()? as u64) << 8;
                            elapsed_cycles |= (device.read_byte()? as u64) << 16;
                            elapsed_cycles |= (device.read_byte()? as u64) << 24;
                            elapsed_cycles |= (device.read_byte()? as u64) << 32;
                            elapsed_cycles |= (device.read_byte()? as u64) << 40;
                            elapsed_cycles |= (device.read_byte()? as u64) << 48;
                            elapsed_cycles |= (device.read_byte()? as u64) << 56;
                            writeln!(&mut stdout, "      elapsed cycles: {}", elapsed_cycles)?;

                            // Copy rasterizer memory back to tile
                            writeln!(&mut stdout, "    read tile")?;
                            device.write_byte(0x03)?;
                            for y in 0..TILE_DIM as usize {
                                for x in 0..TILE_DIM as usize {
                                    let buffer_index = (HEIGHT - 1 - (tile_min_y as usize + y)) * WIDTH + tile_min_x as usize + x;
                                    back_buffer[buffer_index] = device.read_u32()?;
                                }
                            }
                        }
                    }
                }

                window.update_with_buffer(&back_buffer, WIDTH, HEIGHT).unwrap();

                writeln!(&mut stdout, "frame complete")?;
                device.write_byte(0x05)?;

                stdout.reset()?;
            }
            command_byte => {
                return Err(format!("Invalid UART command byte received: 0x{:02x}", command_byte).into());
            }
        }
    }
}
