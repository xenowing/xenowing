#![feature(default_alloc_error_handler)]
#![no_main]
#![no_std]

extern crate alloc;

use xw::{marv, stdio, uart};

use alloc::boxed::Box;
use alloc::vec::Vec;

use core::fmt::Write;
use core::ptr;

#[no_mangle]
fn main() -> ! {
    /*writeln!(stdio::stdout(), "Hello from the program!").unwrap();

    {
        let lol = Box::new(0xfadebabeu32);
        writeln!(stdio::stdout(), "lol: 0x{:08x}", *lol).unwrap();
    }

    {
        let lol = Box::new(0xdeadbeefu32);

        let mut my_vec = Vec::new();
        writeln!(stdio::stdout(), "my_vec: {:?}", my_vec).unwrap();
        let my_vec_cycles_start = marv::cycles();
        for i in 0..100 {
            my_vec.push(i);
        }
        let my_vec_cycles = marv::cycles() - my_vec_cycles_start;
        writeln!(stdio::stdout(), "my_vec: {:?}", my_vec).unwrap();
        writeln!(stdio::stdout(), "populated in {} cycles", my_vec_cycles).unwrap();

        #[derive(Debug)]
        #[repr(align(256))]
        struct Something {
            a: u32,
            b: u32,
            c: u64,
            d: u128,
        }

        let mut my_vec_2 = Vec::new();
        writeln!(stdio::stdout(), "my_vec_2: {:?}", my_vec_2).unwrap();
        let my_vec_2_cycles_start = marv::cycles();
        for i in 0..20 {
            my_vec_2.push(Something {
                a: i,
                b: i * 2,
                c: i as u64 * 0xaa,
                d: 0xfadebabedeadbeefafedbabeabad1dea,
            });
        }
        let my_vec_2_cycles = marv::cycles() - my_vec_2_cycles_start;
        writeln!(stdio::stdout(), "my_vec_2: {:?}", my_vec_2).unwrap();
        writeln!(stdio::stdout(), "populated in {} cycles", my_vec_2_cycles).unwrap();

        writeln!(stdio::stdout(), "lol: 0x{:08x}", *lol).unwrap();
    }

    writeln!(stdio::stdout(), "Yay no crash").unwrap();*/

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
                    unsafe {
                        ptr::write_volatile(addr as _, data);
                    }
                }
                0x01 => {
                    // Read word
                    let addr = uart::read_u32_le();
                    let data = unsafe { ptr::read_volatile(addr as *const u32) };
                    uart::write_u32_le(data);
                }
                0x02 => {
                    // Write tile
                    let base_addr = 0x04000000 as *mut u32; // TODO: Proper constant
                    for i in 0..256 { // TODO: Proper constant
                        let pixel = uart::read_u32_le();
                        unsafe {
                            ptr::write_volatile(base_addr.offset(i), pixel);
                        }
                    }
                }
                0x03 => {
                    // Read tile
                    let base_addr = 0x04000000 as *mut u32; // TODO: Proper constant
                    for i in 0..256 { // TODO: Proper constant
                        let pixel = unsafe { ptr::read_volatile(base_addr.offset(i)) };
                        uart::write_u32_le(pixel);
                    }
                }
                0x04 => {
                    // Rasterize
                    let start_cycles = marv::cycles();

                    unsafe {
                        // TODO: Proper constants/values
                        ptr::write_volatile(0x03000000 as *mut u32, 1);
                        while ptr::read_volatile(0x03000000 as *mut u32) != 0 {
                            // Do nothing
                        }
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
