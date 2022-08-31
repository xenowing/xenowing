mod model_device;
mod model_environment;
mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}
mod sim_device;

use model_environment::*;

use abstract_device::*;

use strugl::*;

use strugl_test::*;

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

use std::env;

fn main() {
    let device_type = env::args().skip(1).nth(0).expect("No device type argument provided");

    let mut device: Box<dyn Device> = match device_type.as_str() {
        "model" => Box::new(model_device::ModelDevice::new()),
        "sim" => Box::new(sim_device::SimDevice::new()),
        _ => panic!("Invalid device type argument")
    };

    let mut window = Window::new("strugl", WIDTH as _, HEIGHT as _, WindowOptions {
        scale: Scale::X2,
        scale_mode: ScaleMode::AspectRatioStretch,
        ..WindowOptions::default()
    }).unwrap();

    let mut c = Context::new(&mut *device);
    let env = ModelEnvironment;
    let mut strugl_test = StruglTest::new(&mut c, &env);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        strugl_test.render_frame(&mut c, &env);

        let mut flipped_buffer: Vec<u32> = Vec::with_capacity(c.back_buffer.len());
        for y in (0..HEIGHT).rev() {
            flipped_buffer.extend_from_slice(&c.back_buffer[(y * WIDTH) as usize..((y + 1) * WIDTH) as usize]);
        }
        window.update_with_buffer(&flipped_buffer, WIDTH as _, HEIGHT as _).unwrap();
    }
}
