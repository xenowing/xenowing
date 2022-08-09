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
    for i in 0..4 {
        ret |= (read_u8() as u32) << i * 8;
    }
    ret
}

pub fn read_u64_le() -> u64 {
    let mut ret = 0;
    for i in 0..2 {
        ret |= (read_u32_le() as u64) << i * 32;
    }
    ret
}

pub fn read_u128_le() -> u128 {
    let mut ret = 0;
    for i in 0..2 {
        ret |= (read_u64_le() as u128) << i * 64;
    }
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
    for i in 0..4 {
        write_u8((x >> (i * 8)) as _);
    }
}

pub fn write_u64_le(x: u64) {
    for i in 0..2 {
        write_u32_le((x >> (i * 32)) as _);
    }
}

pub fn write_u128_le(x: u128) {
    for i in 0..2 {
        write_u64_le((x >> (i * 64)) as _);
    }
}
