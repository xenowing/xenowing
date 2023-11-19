#![no_main]
#![no_std]

use xw::stdio;

use core::fmt::Write;

#[no_mangle]
fn main() -> ! {
    writeln!(stdio::stdout(), "Hello from the bootloader asset!").unwrap();
    writeln!(stdio::stdout()).unwrap();
    writeln!(
        stdio::stdout(),
        "Lots and lots of things have to work for this to show up, so great job buddy! :)"
    )
    .unwrap();

    loop {}
}
