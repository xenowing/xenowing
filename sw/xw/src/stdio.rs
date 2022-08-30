use crate::uart::*;

use core::fmt::{Result, Write};

// TODO: This is specific to xw-blaster, and may have a better home
const COMMAND_PUTC: u8 = 0x00;

pub fn putc(c: char) {
    write_u8(COMMAND_PUTC);
    // TODO: This shouldn't actually be safe... :)
    write_u8(c as _);
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
