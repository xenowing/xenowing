#![feature(default_alloc_error_handler)]
#![no_main]
#![no_std]

mod native_device;

use native_device::*;

use color_thrust_interface::device::*;
use color_thrust_interface::params_and_regs::*;

use xw::{marv, stdio, uart};

use core::fmt::Write;

#[no_mangle]
fn main() -> ! {
    let mut device = NativeDevice::new();

    writeln!(stdio::stdout(), "ready for commands").unwrap();

    loop {
        // TODO: Proper command
        uart::write_u8(0x02);

        loop {
            // TODO: Proper command
            match uart::read_u8() {
                0x00 => {
                    // Write word
                    let addr = uart::read_u32_le();
                    let data = uart::read_u32_le();
                    let base_addr = 0x03000000; // TODO: Proper constant
                    device.write_reg(addr.wrapping_sub(base_addr), data);
                }
                0x01 => {
                    // Read word
                    let addr = uart::read_u32_le();
                    let base_addr = 0x03000000; // TODO: Proper constant
                    let data = device.read_reg(addr.wrapping_sub(base_addr));
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
                command => {
                    panic!("unrecognized command: 0x{:02x}", command);
                }
            }
        }
    }
}
