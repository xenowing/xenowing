#![feature(default_alloc_error_handler)]
#![no_main]
#![no_std]

extern crate alloc;

mod native_device;
mod native_environment;

use native_device::*;
use native_environment::*;

use abstract_environment::*;

use strugl::*;

use strugl_test::*;

use xw::{stdio, uart};

use core::fmt::Write;

#[no_mangle]
fn main() -> ! {
    let mut c = Context::new(NativeDevice::new());
    let env = NativeEnvironment;
    let mut strugl_test = StruglTest::new(&mut c, &env);

    writeln!(stdio::stdout(), "ready for commands").unwrap();

    loop {
        // TODO: Proper command
        uart::write_u8(0x02);

        loop {
            // TODO: Proper command
            match uart::read_u8() {
                0x00 => {
                    // Render and transmit entire frame via strugl
                    let start_cycles = env.cycles();

                    strugl_test.render_frame(&mut c, &env);

                    let end_cycles = env.cycles();
                    let elapsed_cycles = end_cycles - start_cycles;

                    // TODO: Proper command
                    uart::write_u8(0x03);
                    uart::write_u64_le(elapsed_cycles);

                    let back_buffer = c.extract_back_buffer();
                    for y in 0..HEIGHT {
                        for x in 0..WIDTH {
                            uart::write_u32_le(back_buffer[(y * WIDTH + x) as usize]);
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
