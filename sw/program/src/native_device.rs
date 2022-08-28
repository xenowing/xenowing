use color_thrust_interface::device::*;

use core::ptr;

// TODO: Make singleton somehow?
// TODO: Phantom data for enforcing some kind of lifetime?
pub struct NativeDevice;

impl NativeDevice {
    pub fn new() -> NativeDevice {
        NativeDevice
    }
}

impl Device for NativeDevice {
    fn write_reg(&mut self, addr: u32, data: u32) {
        let base_addr = 0x03000000 as *mut u32; // TODO: Proper constant
        unsafe {
            ptr::write_volatile(base_addr.offset((addr * 4) as _) as _, data);
        }
    }

    fn read_reg(&mut self, addr: u32) -> u32 {
        let base_addr = 0x03000000 as *mut u32; // TODO: Proper constant
        unsafe { ptr::read_volatile(base_addr.offset((addr * 4) as _) as *const u32) }
    }

    fn write_color_buffer_word(&mut self, addr: u32, data: u128) {
        let base_addr = 0x04000000 as *mut u128; // TODO: Proper constant
        unsafe {
            ptr::write_volatile(base_addr.offset(addr as _), data);
        }
    }

    fn read_color_buffer_word(&mut self, addr: u32) -> u128 {
        let base_addr = 0x04000000 as *mut u128; // TODO: Proper constant
        unsafe { ptr::read_volatile(base_addr.offset(addr as _)) }
    }

    fn write_depth_buffer_word(&mut self, addr: u32, data: u128) {
        let base_addr = 0x05000000 as *mut u128; // TODO: Proper constant
        unsafe {
            ptr::write_volatile(base_addr.offset(addr as _), data);
        }
    }

    fn read_depth_buffer_word(&mut self, addr: u32) -> u128 {
        let base_addr = 0x05000000 as *mut u128; // TODO: Proper constant
        unsafe { ptr::read_volatile(base_addr.offset(addr as _)) }
    }

    fn write_tex_buffer_word(&mut self, addr: u32, data: u128) {
        let addr = addr as *mut u128;
        unsafe {
            ptr::write_volatile(addr, data);
        }
    }
}
