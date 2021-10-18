use core::ptr;

#[repr(C)]
struct Regs {
    tx_status: u8, _padding0: [u8; 15],
    tx_write: u8, _padding1: [u8; 15],

    rx_status: u8, _padding2: [u8; 15],
    rx_read: u8, _padding3: [u8; 15],
}

const REGS: *mut Regs = 0x03000000 as _;

pub fn read() -> u8 {
    unsafe {
        while (ptr::read_volatile(&(*REGS).rx_status) & 1) == 0 {
            // Do nothing
        }

        ptr::read_volatile(&(*REGS).rx_read)
    }
}

pub fn write(x: u8) {
    unsafe {
        while (ptr::read_volatile(&(*REGS).tx_status) & 1) == 0 {
            // Do nothing
        }

        ptr::write_volatile(&mut (*REGS).tx_write, x);
    }
}
