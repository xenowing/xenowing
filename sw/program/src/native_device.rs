use abstract_device::*;

use alloc::alloc::{alloc, Layout};

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
    fn mem_alloc(&mut self, num_words: u32, align_words: u32) -> u32 {
        unsafe {
            alloc(
                Layout::from_size_align((num_words * 16) as _, (align_words * 16) as _)
                    .expect("Couldn't create memory layout")
            ) as _
        }
    }

    fn mem_dealloc(&mut self, _addr: u32) {
        todo!()
    }

    fn mem_write_word(&mut self, addr: u32, data: u128) {
        let addr = addr as *mut u128;
        unsafe {
            ptr::write_volatile(addr, data);
        }
    }

    fn mem_read_word(&mut self, addr: u32) -> u128 {
        let addr = addr as *const u128;
        unsafe { ptr::read_volatile(addr) }
    }

    fn bit_pusher_write_reg(&mut self, addr: u32, data: u32) {
        let base_addr = 0x06000000 as *mut u32; // TODO: Proper constant
        unsafe {
            ptr::write_volatile(base_addr.offset((addr * 4) as _) as _, data);
        }
    }

    fn bit_pusher_read_reg(&mut self, addr: u32) -> u32 {
        let base_addr = 0x06000000 as *const u32; // TODO: Proper constant
        unsafe { ptr::read_volatile(base_addr.offset((addr * 4) as _) as *const u32) }
    }

    fn color_thrust_write_reg(&mut self, addr: u32, data: u32) {
        let base_addr = 0x03000000 as *mut u32; // TODO: Proper constant
        unsafe {
            ptr::write_volatile(base_addr.offset((addr * 4) as _) as _, data);
        }
    }

    fn color_thrust_read_reg(&mut self, addr: u32) -> u32 {
        let base_addr = 0x03000000 as *const u32; // TODO: Proper constant
        unsafe { ptr::read_volatile(base_addr.offset((addr * 4) as _) as *const u32) }
    }

    fn color_thrust_write_color_buffer_word(&mut self, addr: u32, data: u128) {
        let base_addr = 0x04000000 as *mut u128; // TODO: Proper constant
        unsafe {
            ptr::write_volatile(base_addr.offset(addr as _), data);
        }
    }

    fn color_thrust_read_color_buffer_word(&mut self, addr: u32) -> u128 {
        let base_addr = 0x04000000 as *const u128; // TODO: Proper constant
        unsafe { ptr::read_volatile(base_addr.offset(addr as _)) }
    }

    fn color_thrust_write_depth_buffer_word(&mut self, addr: u32, data: u128) {
        let base_addr = 0x05000000 as *mut u128; // TODO: Proper constant
        unsafe {
            ptr::write_volatile(base_addr.offset(addr as _), data);
        }
    }

    fn color_thrust_read_depth_buffer_word(&mut self, addr: u32) -> u128 {
        let base_addr = 0x05000000 as *const u128; // TODO: Proper constant
        unsafe { ptr::read_volatile(base_addr.offset(addr as _)) }
    }
}
