#![feature(default_alloc_error_handler)]
#![no_main]
#![no_std]

use xw::{leds, uart, stdio};

use core::fmt::Write;
use core::mem;

#[no_mangle]
fn main() -> ! {
    writeln!(stdio::stdout(), "xw online").unwrap();

    extern "C" {
        static mut _sprogram: u8;
        static _max_program_size: u8;
    }

    // TODO: Proper command
    uart::write_u8(0x01);
    // TODO: Proper filename
    let filename = "../../program/target/program.bin";
    for b in filename.bytes() {
        uart::write_u8(b);
    }
    uart::write_u8(0);
    let program_size = uart::read_u32_le();
    // TODO: Is there a better way to get this symbol value?
    let max_program_size = unsafe { &_max_program_size as *const _ as u32 };
    if program_size > max_program_size {
        panic!("program size ({} bytes) must not be larger than {} bytes", program_size, max_program_size);
    }

    let program_ram = unsafe { &mut _sprogram } as *mut u8;
    for i in 0..program_size {
        let b = uart::read_u8();
        leds::set(b);
        unsafe {
            *program_ram.offset(i as _) = b;
        }
    }

    writeln!(stdio::stdout(), "program read successful").unwrap();

    let program_entry = unsafe {
        mem::transmute::<_, extern "C" fn() -> !>(program_ram)
    };
    program_entry()
}
