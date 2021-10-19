use core::ptr;

#[repr(C)]
struct Regs {
    tx_status: u8, _padding0: [u8; 15],
    tx_write: u8, _padding1: [u8; 15],

    rx_status: u8, _padding2: [u8; 15],
    rx_read: u8, _padding3: [u8; 15],
}

const REGS: *mut Regs = 0x02000000 as _;

pub fn read_u8() -> u8 {
    unsafe {
        while (ptr::read_volatile(&(*REGS).rx_status) & 1) == 0 {
            // Do nothing
        }

        ptr::read_volatile(&(*REGS).rx_read)
    }
}

pub fn read_u32_le() -> u32 {
    let mut ret = 0;
    ret |= (read_u8() as u32) << 0;
    ret |= (read_u8() as u32) << 8;
    ret |= (read_u8() as u32) << 16;
    ret |= (read_u8() as u32) << 24;
    ret
}

pub fn write_u8(x: u8) {
    unsafe {
        while (ptr::read_volatile(&(*REGS).tx_status) & 1) == 0 {
            // Do nothing
        }

        ptr::write_volatile(&mut (*REGS).tx_write, x);
    }
}

pub fn write_u32_le(x: u32) {
    write_u8((x >> 0) as _);
    write_u8((x >> 8) as _);
    write_u8((x >> 16) as _);
    write_u8((x >> 24) as _);
}

pub fn write_u64_le(x: u64) {
    write_u32_le((x >> 0) as _);
    write_u32_le((x >> 32) as _);
}
