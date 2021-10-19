use core::ptr;

pub fn set(leds: u8) {
    const LEDS: *mut u8 = 0x01000000 as _;

    unsafe {
        ptr::write_volatile(LEDS, leds);
    }
}
