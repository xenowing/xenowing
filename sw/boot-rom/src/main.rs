#![no_main]
#![no_std]

use xw::{leds, uart, stdio};

use core::fmt::Write;
use core::mem;

#[no_mangle]
extern "C" fn main() -> ! {
    writeln!(stdio::stdout(), "xw online - and now oxidized!").unwrap();

    writeln!(stdio::stdout(), "heap start: 0x{:08x}", xw::heap::heap_start() as u32).unwrap();
    writeln!(stdio::stdout(), "heap end:   0x{:08x}", xw::heap::heap_end() as u32).unwrap();

    // TODO: Proper command
    uart::write(0x01);
    // TODO: Proper filename
    let filename = "../../program/program.bin";
    for b in filename.bytes() {
        uart::write(b);
    }
    uart::write(0);
    let mut len = 0;
    len |= (uart::read() as u32) << 0;
    len |= (uart::read() as u32) << 8;
    len |= (uart::read() as u32) << 16;
    len |= (uart::read() as u32) << 24;

    let program_ram = 0x01000000 as *mut u8;
    for i in 0..len {
        let b = uart::read();
        leds::set(b);
        unsafe {
            *program_ram.offset(i as _) = b;
        }
    }

    writeln!(stdio::stdout(), "program RAM read successful").unwrap();

    let program_ram_entry = unsafe {
        mem::transmute::<_, extern "C" fn() -> !>(program_ram)
    };
    program_ram_entry()
}
