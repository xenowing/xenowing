use rtl_meta::shovel::char_display::*;

use core::fmt::{Result, Write};
use core::ptr;

const MAP: *mut u8 = 0x01000000 as _;

static mut MAP_OFFSET: u32 = 0;

pub fn putc(c: char) {
    unsafe {
        if c == '\n' {
            MAP_OFFSET += CHARS_WIDTH;
            while (MAP_OFFSET % CHARS_WIDTH) != 0 {
                MAP_OFFSET -= 1;
            }
        } else {
            // TODO: This shouldn't actually be safe... :)
            let c = (c as u8) - 32;
            ptr::write_volatile(MAP.offset((MAP_OFFSET * 16) as _), c);
            MAP_OFFSET += 1;
        }

        while MAP_OFFSET == CHARS_WIDTH * CHARS_HEIGHT {
            for y in 0..CHARS_HEIGHT - 1 {
                for x in 0..CHARS_WIDTH {
                    let c = ptr::read_volatile(MAP.offset((((y + 1) * CHARS_WIDTH + x) * 16) as _));
                    ptr::write_volatile(MAP.offset(((y * CHARS_WIDTH + x) * 16) as _), c);
                }
            }
            for x in 0..CHARS_WIDTH {
                ptr::write_volatile(MAP.offset((x * 16) as _), 0);
            }

            MAP_OFFSET -= CHARS_WIDTH;
        }
    }
}

pub fn puts(s: &str) {
    puts_nn(s);
    putc('\n');
}

pub fn puts_nn(s: &str) {
    for c in s.chars() {
        putc(c);
    }
}

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        puts_nn(s);

        Ok(())
    }
}

pub fn stdout() -> Stdout {
    Stdout
}
