mod bit_pusher;
mod color_thrust;

use bit_pusher::*;
use color_thrust::*;

use crate::mem_allocator::*;

use abstract_device::*;

pub struct ModelDevice {
    bit_pusher: BitPusher,
    color_thrust: ColorThrust,

    mem: Box<[u128]>,
    mem_allocator: MemAllocator,
}

impl ModelDevice {
    pub fn new() -> ModelDevice {
        ModelDevice {
            bit_pusher: BitPusher::new(),
            color_thrust: ColorThrust::new(),

            mem: vec![0; MEM_NUM_WORDS as usize].into_boxed_slice(),
            mem_allocator: MemAllocator::new(),
        }
    }
}

impl Device for ModelDevice {
    fn mem_alloc(&mut self, num_words: u32, align_words: u32) -> u32 {
        self.mem_allocator.alloc(num_words, align_words)
    }

    fn mem_dealloc(&mut self, addr: u32) {
        self.mem_allocator.dealloc(addr);
    }

    fn mem_write_word(&mut self, addr: u32, data: u128) {
        if (addr % 16) != 0 {
            panic!("Unaligned device memory access");
        }
        self.mem[(addr / 16) as usize] = data;
    }

    fn mem_read_word(&mut self, addr: u32) -> u128 {
        if (addr % 16) != 0 {
            panic!("Unaligned device memory access");
        }
        self.mem[(addr / 16) as usize]
    }

    fn bit_pusher_write_reg(&mut self, addr: u32, data: u32) {
        self.bit_pusher.write_reg(addr, data, &mut self.mem, &mut self.color_thrust);
    }

    fn bit_pusher_read_reg(&mut self, addr: u32) -> u32 {
        self.bit_pusher.read_reg(addr)
    }

    fn color_thrust_write_reg(&mut self, addr: u32, data: u32) {
        self.color_thrust.write_reg(addr, data, &self.mem);
    }

    fn color_thrust_read_reg(&mut self, addr: u32) -> u32 {
        self.color_thrust.read_reg(addr)
    }

    fn color_thrust_write_color_buffer_word(&mut self, addr: u32, data: u128) {
        self.color_thrust.write_color_buffer_word(addr, data);
    }

    fn color_thrust_read_color_buffer_word(&mut self, addr: u32) -> u128 {
        self.color_thrust.read_color_buffer_word(addr)
    }

    fn color_thrust_write_depth_buffer_word(&mut self, addr: u32, data: u128) {
        self.color_thrust.write_depth_buffer_word(addr, data);
    }

    fn color_thrust_read_depth_buffer_word(&mut self, addr: u32) -> u128 {
        self.color_thrust.read_depth_buffer_word(addr)
    }
}
