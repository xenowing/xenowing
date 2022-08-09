#![no_main]
#![no_std]

extern crate alloc;

#[macro_use]
extern crate static_assertions;

pub mod leds;
mod heap;
pub mod marv;
pub mod stdio;
pub mod uart;

mod asm {
    use core::arch::global_asm;

    global_asm!(include_str!("_cycles.s"));
    global_asm!(include_str!("entry.s"));
}

use core::fmt::Write;
use core::panic::PanicInfo;

#[no_mangle]
extern "C" fn _rust_entry() -> ! {
    extern "Rust" {
        fn main() -> !;
    }

    // Reset hw state for soft resets
    leds::set(0x00);
    heap::init();

    unsafe {
        main();
    }
}

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    leds::set(0xff);

    writeln!(stdio::stdout(), "Panic: {}", panic_info).ok();

    loop {
        marv::sleep_cycles(100000000 / 4);
        leds::set(0x00);
        marv::sleep_cycles(100000000 / 4);
        leds::set(0xff);
    }
}
