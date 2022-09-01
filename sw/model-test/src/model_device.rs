mod color_thrust;

use color_thrust::*;

use abstract_device::*;

pub struct ModelDevice {
    color_thrust: ColorThrust,
}

impl ModelDevice {
    pub fn new() -> ModelDevice {
        ModelDevice {
            color_thrust: ColorThrust::new(),
        }
    }
}

impl Device for ModelDevice {
    fn mem_alloc(&mut self, _num_words: u32, _align_words: u32) -> u32 {
        todo!()
    }

    fn mem_dealloc(&mut self, _addr: u32) {
        todo!()
    }

    fn mem_write_word(&mut self, addr: u32, data: u128) {
        todo!()
    }

    fn mem_read_word(&mut self, addr: u32) -> u128 {
        todo!()
    }

    fn color_thrust_write_reg(&mut self, addr: u32, data: u32) {
        self.color_thrust.write_reg(addr, data);
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
