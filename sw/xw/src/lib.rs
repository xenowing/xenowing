#![feature(global_asm)]
#![no_main]
#![no_std]

// TODO: Remove/refactor once we figure out how to actually alloc
pub mod heap {
    #[inline]
    pub fn heap_start() -> *mut u32 {
        extern "C" {
            static mut _sheap: u32;
        }

        unsafe { &mut _sheap }
    }

    #[inline]
    pub fn heap_end() -> *mut u32 {
        extern "C" {
            static mut _eheap: u32;
        }

        unsafe { &mut _eheap }
    }
}

pub mod leds;
pub mod marv;
pub mod stdio;
pub mod uart;

mod asm {
    global_asm!(include_str!("_cycles.s"));
    global_asm!(include_str!("entry.s"));
}

use core::fmt::Write;
use core::panic::PanicInfo;

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
